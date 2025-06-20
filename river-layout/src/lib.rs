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

use wayland_client::ConnectError;
use wayland_client::Connection;
use wayland_client::Dispatch;
use wayland_client::QueueHandle;
use wayland_client::globals::BindError;
use wayland_client::globals::GlobalError;
use wayland_client::globals::GlobalListContents;
use wayland_client::globals::registry_queue_init;

use wayland_client::protocol::wl_output;
use wayland_client::protocol::wl_registry;

use protocol::river_layout_manager_v3;
use protocol::river_layout_v3;

#[derive(Debug)]
pub struct GeneratedLayout {
    views: Vec<ViewDimensions>,
}

#[derive(Debug)]
pub struct ViewDimensions {
    location: (i32, i32),
    space: (u32, u32),
}

pub trait LayoutGenerator: Sized + 'static {
    const NAMESPACE: &'static str;
    type Err: std::error::Error;

    fn cmd(&mut self, tags: Option<u32>, output: &str, cmd: &str) -> Result<(), Self::Err>;
    fn generate_layout(
        &mut self,
        tags: u32,
        output: &str,
        space: (u32, u32),
    ) -> Result<GeneratedLayout, Self::Err>;

    fn run(self) -> Result<(), Error> {
        let conn = Connection::connect_to_env()?;
        let (globals, queue) = registry_queue_init::<State<Self>>(&conn)?;
        let river_layout_manager_v3 = globals.bind(&queue.handle(), 1..=2, ())?;

        let state = State {
            river_layout_manager_v3,
            layout_generator: self,
            tags: None,
            error: None,
        };

        'event_loop: loop {
            if let Some(ref err) = state.error {
                match err {
                    _ => break 'event_loop,
                }
            }
        }

        state.error.map_or(Ok(()), Err)
    }
}

#[derive(Debug)]
pub enum Error {
    Connect(ConnectError),
    Global(GlobalError),
    Bind(BindError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Error::Connect(err) => write!(f, "{err}"),
            Error::Global(err) => write!(f, "{err}"),
            Error::Bind(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<ConnectError> for Error {
    fn from(err: ConnectError) -> Error {
        Error::Connect(err)
    }
}

impl From<GlobalError> for Error {
    fn from(err: GlobalError) -> Error {
        Error::Global(err)
    }
}

impl From<BindError> for Error {
    fn from(err: BindError) -> Error {
        Error::Bind(err)
    }
}

#[derive(Debug)]
pub struct State<LG: LayoutGenerator> {
    river_layout_manager_v3: river_layout_manager_v3::RiverLayoutManagerV3,
    layout_generator: LG,
    tags: Option<u32>,
    error: Option<Error>,
}

impl<LG: LayoutGenerator> Dispatch<wl_registry::WlRegistry, GlobalListContents> for State<LG> {
    fn event(
        _state: &mut State<LG>,
        proxy: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _udata: &GlobalListContents,
        _conn: &Connection,
        qh: &QueueHandle<State<LG>>,
    ) {
        use wl_registry::Event;

        match event {
            Event::Global {
                name,
                interface,
                version,
            } if interface == "wl_output" => {
                let _wl_output: wl_output::WlOutput = proxy.bind(name, version, qh, OutputData {});
            }
            _ => (),
        }
    }
}

#[derive(Debug)]
struct OutputData {}

impl<LG: LayoutGenerator> Dispatch<wl_output::WlOutput, OutputData> for State<LG> {
    fn event(
        state: &mut State<LG>,
        proxy: &wl_output::WlOutput,
        event: wl_output::Event,
        _udata: &OutputData,
        _conn: &Connection,
        qh: &QueueHandle<State<LG>>,
    ) {
        use wl_output::Event;

        match event {
            Event::Name { name } => {
                let _river_layout_v3 = state.river_layout_manager_v3.get_layout(
                    &proxy,
                    LG::NAMESPACE.into(),
                    qh,
                    LayoutData { output_name: name },
                );
            }
            _ => (),
        }
    }
}

#[derive(Debug)]
struct LayoutData {
    output_name: String,
}

impl<LG: LayoutGenerator> Dispatch<river_layout_manager_v3::RiverLayoutManagerV3, ()>
    for State<LG>
{
    fn event(
        _state: &mut State<LG>,
        _proxy: &river_layout_manager_v3::RiverLayoutManagerV3,
        _event: river_layout_manager_v3::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<State<LG>>,
    ) {
    }
}

impl<LG: LayoutGenerator> Dispatch<river_layout_v3::RiverLayoutV3, LayoutData> for State<LG> {
    fn event(
        _state: &mut State<LG>,
        _proxy: &river_layout_v3::RiverLayoutV3,
        _event: river_layout_v3::Event,
        _udata: &LayoutData,
        _conn: &Connection,
        _qh: &QueueHandle<State<LG>>,
    ) {
    }
}
