use actix::prelude::*;

#[derive(Message, Copy, Clone)]
#[rtype(result = "()")]
pub struct Deploy;
