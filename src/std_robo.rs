use cgmath::Vector2;
use robo::*;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum UserCmd {
    GetPos,
    GetState,
    Log(String),
    SetVel(SerVector2<f64>),
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum SysCmd {
    Damage(f64),
    RaiseEvent(Event),
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Cmd {
    UserCmd(UserCmd),
    SysCmd(SysCmd),
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Resp {
    PosIs(SerVector2<f64>),
    StateIs(StdRobo),
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Init,
    Tick,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Output {
    Resp(Resp),
    Event(Event),
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct StdRobo {
    pub health: f64,
    pub pos: SerVector2<f64>,
    pub vel: SerVector2<f64>,
}

/// Used to simplify code when handling commands that don't result in a
/// response.
fn async(_: ()) -> Option<Output> { None }

fn respond(r: Resp) -> Option<Output> { Some(Output::Resp(r)) }

fn raise_event(e: Event) -> Option<Output> { Some(Output::Event(e)) }

impl AsyncRobo for StdRobo {
    type Input = Cmd;
    type Output = Output;

    fn handle_input(&mut self, cmd: Cmd) -> Option<Output> {
        use self::Cmd::*;
        use self::UserCmd::*;
        use self::SysCmd::*;
        use self::Resp::*;

        match cmd {
            UserCmd(cmd) => match cmd {
                GetPos => respond(PosIs(self.pos)),
                GetState => respond(StateIs(self.clone())),
                Log(msg) => async(println!("{}", msg)),
                SetVel(v) => async(self.vel = v),
            },
            SysCmd(cmd) => match cmd {
                RaiseEvent(e) => {
                    if let Event::Tick = e {
                        self.pos.0 += self.vel.0;
                    }

                    raise_event(e)
                },
                Damage(dmg) => async(self.health -= dmg),
            },
        }
    }
}

// =============================================================================
//  Serialize / Deserialize instances
// =============================================================================

use serde;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use std::ops::{Deref, DerefMut};

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct SerVector2<T>(pub Vector2<T>);

impl<T> Deref for SerVector2<T> {
    type Target = Vector2<T>;

    fn deref(&self) -> &Vector2<T> {
        &self.0
    }
}

impl<T> DerefMut for SerVector2<T> {
    fn deref_mut(&mut self) -> &mut Vector2<T> {
        &mut self.0
    }
}

impl<T> Serialize for SerVector2<T> where T: Serialize {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        let tp = (&self.0.x, &self.0.y);
        serializer.serialize_tuple_struct("Vector2", serde::ser::impls::TupleVisitor2::new(&tp))
    }
}

impl<T> Deserialize for SerVector2<T> where T: Deserialize {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
        deserializer.deserialize_tuple_struct("Vector2", 2, serde::de::impls::TupleVisitor2::new())
            .map(|(x, y)| SerVector2(Vector2::new(x, y)))
    }
}
