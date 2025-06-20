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
use wayland_client::globals::GlobalError;
use wayland_client::globals::GlobalListContents;
use wayland_client::globals::registry_queue_init;

use wayland_client::protocol::wl_registry;

pub trait LayoutGenerator: Sized + 'static {
    const NAMESPACE: &'static str;
    type Err: std::error::Error;

    fn run(self) -> Result<(), Error> {
        let conn = Connection::connect_to_env()?;
        let (_globals, _queue) = registry_queue_init::<State>(&conn)?;

        let state = State {
            _tags: None,
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
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Error::Connect(err) => write!(f, "{err}"),
            Error::Global(err) => write!(f, "{err}"),
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

#[derive(Debug)]
pub struct State {
    _tags: Option<u32>,
    error: Option<Error>,
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _state: &mut State,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _udata: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<State>,
    ) {
        todo!()
    }
}
