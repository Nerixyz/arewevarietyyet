use std::time::{Duration, Instant};
use actix::{Actor, ActorFutureExt, AsyncContext, Context, Handler, Message, ResponseFuture, WrapFuture};
use actix::fut::ready;
use regex::internal::Inst;
use crate::model::StreamerModel;
use crate::sullygnome;

const CACHE_TIME: Duration = Duration::from_secs(10 * 60);

pub struct DataActor {
    model: Option<(Instant, StreamerModel)>,
}

impl Actor for DataActor {
    type Context = Context<Self>;
}

pub struct GetJson;

impl Message for GetJson {
    type Result = anyhow::Result<String>;
}

impl Handler<GetJson> for DataActor {
    type Result = ResponseFuture<<GetJson as Message>::Result>;

    fn handle(&mut self, _: GetJson, ctx: &mut Self::Context) -> Self::Result {
        match self.model {
            None | Some((instant)) if Instant::now() - instant > CACHE_TIME => {
              ctx.wait(sullygnome::request().into_actor(self).map(|res, this, _ctx| {
                  match res {
                      Ok(res) => {
                          let json = serde_json::to_string(&model).map_err(anyhow::Error::from);
                          this.model = Some((Instant::now(), res));
                          json
                      },
                      Err(e) => {

                      }
                  }
              }))
            },
            Some((_,model)) => ready(serde_json::to_string(&model).map_err(anyhow::Error::from)),
        }
    }
}