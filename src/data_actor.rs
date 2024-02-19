use crate::{
    model::{StreamerModel, Year},
    sullygnome::{self, GamesResponse, StreamsResponse},
};
use actix::{
    fut::ready, Actor, ActorFuture, ActorFutureExt, AsyncContext, Context, Handler, Message,
    ResponseActFuture, WrapFuture,
};
use anyhow::anyhow;
use chrono::{Datelike, Utc};
use futures::future;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

const CACHE_TIME: Duration = Duration::from_secs(10 * 60);
const FROM_YEAR: i32 = 2018;

pub struct DataActor {
    current_year: Option<(Instant, Arc<StreamerModel>)>,
    prev_years: HashMap<i32, Arc<StreamerModel>>,
    years_n: Arc<Vec<i32>>,
    current_year_n: i32,
}

impl Default for DataActor {
    fn default() -> Self {
        Self {
            current_year: None,
            prev_years: HashMap::new(),
            current_year_n: Utc::now().year(),
            years_n: Arc::new(Vec::new()),
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
        Ok((model, self.years_n.clone()))
    }

    fn put_last_response(&mut self, response: impl Iterator<Item = (i32, Arc<StreamerModel>)>) {
        self.prev_years = HashMap::from_iter(response);
        let mut vec = Vec::from_iter(self.prev_years.keys().copied());
        vec.sort_by(|a, b| b.cmp(a));
        self.years_n = Arc::new(vec);
    }

    fn try_get_cached(&self) -> Option<<GetData as Message>::Result> {
        let (instant, model) = &self.current_year.as_ref()?;
        if Instant::now() - *instant > CACHE_TIME {
            None
        } else {
            Some(Ok((model.clone(), self.years_n.clone())))
        }
    }

    fn get_last_year(
        &self,
        year: i32,
    ) -> impl ActorFuture<Self, Output = <GetData as Message>::Result> {
        let f = (FROM_YEAR.min(self.current_year_n)..self.current_year_n).map(|year| async move {
            let (games, streams) = future::try_join(
                sullygnome::get_all_of::<GamesResponse>(year),
                sullygnome::get_all_of::<StreamsResponse>(year),
            )
            .await?;
            StreamerModel::create(Year::Last(year), games, streams).map(|m| (year, Arc::new(m)))
        });

        future::join_all(f)
            .into_actor(self)
            .map(move |res, this, _| {
                this.put_last_response(res.into_iter().filter_map(Result::ok));
                this.prev_years
                    .get(&year)
                    .cloned()
                    .map(|y| (y, this.years_n.clone()))
                    .ok_or_else(|| anyhow!("No such year exists"))
            })
    }
}

impl Actor for DataActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.spawn(self.get_last_year(0).map(|_, this, _| {
            eprintln!("Loaded {} last year(s)", this.prev_years.len());
        }));
    }
}

pub struct GetData(pub Year);

impl Message for GetData {
    type Result = anyhow::Result<(Arc<StreamerModel>, Arc<Vec<i32>>)>;
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
                            sullygnome::get_all_of::<GamesResponse>(current_year),
                            sullygnome::get_all_of::<StreamsResponse>(current_year),
                        )
                        .into_actor(self)
                        .map(|res, this, _| (this.put_current_response(res))),
                    )
                }
            },
            Year::Last(year) => {
                let current_year = Utc::now().year();
                if self.current_year_n == current_year {
                    Box::pin(ready(
                        self.prev_years
                            .get(&year)
                            .cloned()
                            .map(|y| (y, self.years_n.clone()))
                            .ok_or_else(|| anyhow!("This year isn't tracked")),
                    ))
                } else {
                    // the year changed while we were running
                    self.current_year_n = current_year;
                    Box::pin(self.get_last_year(year))
                }
            }
        }
    }
}
