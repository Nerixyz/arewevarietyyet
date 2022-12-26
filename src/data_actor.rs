use crate::{
    model::{StreamerModel, Year},
    streamcounter::LongestDitch,
    sullygnome,
};
use actix::{
    fut::ready, Actor, ActorFuture, ActorFutureExt, AsyncContext, Context, Handler, Message,
    ResponseActFuture, WrapFuture,
};
use chrono::{Datelike, Utc};
use futures::future;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

const CACHE_TIME: Duration = Duration::from_secs(10 * 60);

pub struct DataActor {
    current_year: Option<(Instant, Arc<StreamerModel>)>,
    last_year: Arc<StreamerModel>,
    current_last_year: i32,
}

impl Default for DataActor {
    fn default() -> Self {
        Self {
            current_year: None,
            last_year: Arc::new(StreamerModel {
                games: vec![],
                total_time_min: 0,
                at_least_one_stream: false,
                variety_percent: 0.0,
                ow_percent: 0.0,
                are_we_variety: false,
                days_ditched: 0,
                days_until_now: 0,
                percent_ditched: 0.0,
                year: 0,
                longest_ditch: LongestDitch::Current { from: Utc::now() },
            }),
            current_last_year: Utc::now().year() - 1,
        }
    }
}

impl DataActor {
    fn put_current_response(
        &mut self,
        response: anyhow::Result<(sullygnome::GamesResponse, sullygnome::StreamsResponse)>,
    ) -> <GetData as Message>::Result {
        let (games, streams) = response?;
        let model = Arc::new(StreamerModel::create(Year::Current, games, streams)?);
        self.current_year = Some((Instant::now(), Arc::clone(&model)));
        Ok(model)
    }

    fn put_last_response(
        &mut self,
        response: anyhow::Result<(sullygnome::GamesResponse, sullygnome::StreamsResponse)>,
    ) -> <GetData as Message>::Result {
        let (games, streams) = response?;
        self.last_year = Arc::new(StreamerModel::create(Year::Last, games, streams)?);
        Ok(self.last_year.clone())
    }

    fn try_get_cached(&self) -> Option<<GetData as Message>::Result> {
        let (instant, model) = &self.current_year.as_ref()?;
        if Instant::now() - *instant > CACHE_TIME {
            None
        } else {
            Some(Ok(model.clone()))
        }
    }

    fn get_last_year(&self) -> impl ActorFuture<Self, Output = <GetData as Message>::Result> {
        let last_year = self.current_last_year;

        future::try_join(
            sullygnome::get_games(last_year),
            sullygnome::get_streams(last_year),
        )
        .into_actor(self)
        .map(|res, this, _| this.put_last_response(res))
    }
}

impl Actor for DataActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.spawn(self.get_last_year().map(|res, _this, _| {
            if let Err(e) = res {
                eprintln!("Failed to get last year's data: {e}");
            }
        }));
    }
}

pub struct GetData(pub Year);

impl Message for GetData {
    type Result = anyhow::Result<Arc<StreamerModel>>;
}

impl Handler<GetData> for DataActor {
    type Result = ResponseActFuture<Self, <GetData as Message>::Result>;

    fn handle(&mut self, GetData(year): GetData, _: &mut Self::Context) -> Self::Result {
        match year {
            Year::Current => match self.try_get_cached() {
                Some(cached) => Box::pin(ready(cached)),
                None => {
                    let current_year = Utc::now().year();
                    Box::pin(
                        future::try_join(
                            sullygnome::get_games(current_year),
                            sullygnome::get_streams(current_year),
                        )
                        .into_actor(self)
                        .map(|res, this, _| (this.put_current_response(res))),
                    )
                }
            },
            Year::Last => {
                let current_year = Utc::now().year();
                if self.current_last_year + 1 == current_year {
                    Box::pin(ready(Ok(self.last_year.clone())))
                } else {
                    // the year changed while we were running
                    self.current_last_year = current_year - 1;
                    Box::pin(self.get_last_year())
                }
            }
        }
    }
}
