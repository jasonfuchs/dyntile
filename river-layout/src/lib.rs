pub mod protocol {
    use wayland_client;
    use wayland_client::protocol::*;

    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("protocol/river-layout-v3.xml");
    }
    use self::__interfaces::*;

    wayland_scanner::generate_client_code!("protocol/river-layout-v3.xml");
}

use wayland_client::Connection;
use wayland_client::Dispatch;
use wayland_client::QueueHandle;
use wayland_client::globals::GlobalListContents;
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::wl_registry;
use wayland_client::protocol::wl_registry::WlRegistry;

use protocol::river_layout_manager_v3;
use protocol::river_layout_manager_v3::RiverLayoutManagerV3;
use protocol::river_layout_v3;
use protocol::river_layout_v3::RiverLayoutV3;

#[derive(Debug)]
pub struct View {
    location: (i32, i32),
    space: (u32, u32),
}

#[derive(Debug)]
pub struct Layout<const VIEW_COUNT: usize> {
    name: Box<str>,
    views: [View; VIEW_COUNT],
}

#[derive(Debug)]
struct Serial(u32);

#[derive(Debug)]
struct State<L: LayoutGenerator> {
    layout_manager: RiverLayoutManagerV3,
    layout_generator: L,
    tags: Option<u32>,
    exit: bool,
}

pub trait LayoutGenerator: Sized + 'static {
    const NAMESPACE: &'static str;
    type Err: std::error::Error;

    fn user_cmd(&mut self, tags: Option<u32>, output: &str, cmd: &str) -> Result<(), Self::Err>;
    fn generate_layout<const VIEW_COUNT: usize>(
        &mut self,
        tags: u32,
        output: &str,
        usable_space: (u32, u32),
    ) -> Result<Layout<VIEW_COUNT>, Self::Err>;

    fn run(self) -> Result<(), Self::Err> {
        let conn = Connection::connect_to_env().unwrap(); // FIXME error handling
        let (globals, queue) = registry_queue_init::<State<Self>>(&conn).unwrap();

        let layout_manager = globals.bind(&queue.handle(), 1..=2, ()).unwrap();

        let state = State {
            layout_manager,
            layout_generator: self,
            tags: None,
            exit: false,
        };

        todo!()
    }
}

impl<L: LayoutGenerator> Dispatch<WlRegistry, GlobalListContents> for State<L> {
    fn event(
        _state: &mut Self,
        _proxy: &WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _q: &QueueHandle<State<L>>,
    ) {
    }
}

impl<L: LayoutGenerator> Dispatch<RiverLayoutManagerV3, ()> for State<L> {
    fn event(
        _state: &mut Self,
        _proxy: &RiverLayoutManagerV3,
        _event: river_layout_manager_v3::Event,
        _data: &(),
        _conn: &Connection,
        _q: &QueueHandle<State<L>>,
    ) {
    }
}

impl<L: LayoutGenerator> Dispatch<RiverLayoutV3, Serial> for State<L> {
    fn event(
        state: &mut Self,
        _proxy: &RiverLayoutV3,
        event: river_layout_v3::Event,
        _data: &Serial,
        _conn: &Connection,
        _q: &QueueHandle<State<L>>,
    ) {
        use river_layout_v3::Event;

        match event {
            Event::NamespaceInUse => state.exit = true,
            Event::LayoutDemand { .. } => (),
            Event::UserCommand { command } => {
                state
                    .layout_generator
                    .user_cmd(state.tags, "", &command)
                    .unwrap();
            }
            Event::UserCommandTags { tags } => state.tags = Some(tags),
        }
    }
}
