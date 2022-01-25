use std::sync::Arc;
use std::time::{Duration, Instant};
use actix::{Actor, ActorFutureExt, Context, Handler, Message, ResponseActFuture, WrapFuture};
use actix::fut::ready;
use crate::model::StreamerModel;
use crate::sullygnome::{GamesResponse, self};

const CACHE_TIME: Duration = Duration::from_secs(10 * 60);

#[derive(Default)]
pub struct DataActor {
    model: Option<(Instant, Arc<StreamerModel>)>,
}

impl DataActor {
    fn put_response(&mut self, response: anyhow::Result<GamesResponse>) -> <GetData as Message>::Result {
        match response {
            Ok(res) => {
                let model = Arc::new(res.try_into()?);
                self.model = Some((Instant::now(), Arc::clone(&model)));
                Ok(model)
            },
            Err(e) => {
                Err(e.into())
            }
        }
    }

    fn try_get_cached(&self) -> Option<<GetData as Message>::Result> {
        let (instant, model) = &self.model.as_ref()?;
        if Instant::now() - *instant > CACHE_TIME {
            None
        } else {
            Some(Ok(model.clone()))
        }
    }
}

impl Actor for DataActor {
    type Context = Context<Self>;
}

pub struct GetData;

impl Message for GetData {
    type Result = anyhow::Result<Arc<StreamerModel>>;
}

impl Handler<GetData> for DataActor {
    type Result = ResponseActFuture<Self, <GetData as Message>::Result>;

    fn handle(&mut self, _: GetData, _: &mut Self::Context) -> Self::Result {
        match self.try_get_cached() {
            Some(cached) => Box::pin(ready(cached)),
            None => Box::pin(sullygnome::request().into_actor(self).map(|res, this, _| (this.put_response(res)))),
        }
    }
}