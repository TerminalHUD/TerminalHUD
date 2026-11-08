use super::*;

mod acquisition;
mod assertions;
mod backend;
mod focus;
mod lifecycle;
mod observation;
mod presentation;
mod world;

pub(super) const TOPMOST: u32 = 1;
pub(super) const CLICK_THROUGH: u32 = 2;
pub(super) const OWNED: u32 = TOPMOST | CLICK_THROUGH;
pub(super) const OTHER_STYLE: u32 = 4;

#[derive(Clone, Copy)]
pub(super) struct Window {
    pub(super) visible: bool,
    pub(super) style: u32,
}

pub(super) struct World {
    pub(super) windows: HashMap<u64, Window>,
    pub(super) foreground: Option<u64>,
    pub(super) next_target: u64,
    pub(super) calls: Vec<String>,
    pub(super) failures: VecDeque<&'static str>,
    pub(super) capabilities: Capabilities,
}

#[derive(Clone)]
pub(super) struct Fake(pub(super) Arc<Mutex<World>>);

pub(super) struct Original {
    pub(super) owned_style: u32,
    pub(super) visible: bool,
}

pub(super) fn setup() -> (Controller<Fake>, Arc<Mutex<World>>) {
    let world = Arc::new(Mutex::new(World {
        windows: HashMap::new(),
        foreground: Some(10),
        next_target: 100,
        calls: Vec::new(),
        failures: VecDeque::new(),
        capabilities: Capabilities::ALL,
    }));
    (Controller::new(Fake(world.clone())), world)
}

pub(super) use assertions::assert_view;
