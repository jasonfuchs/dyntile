use wayland_client::ConnectError;
use wayland_client::DispatchError;

#[derive(Debug)]
pub enum Error {
    Connect(ConnectError),
    Dispatch(DispatchError),
    Other(Box<(dyn std::error::Error + 'static)>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            _ => todo!(),
        }
    }
}

impl std::error::Error for Error {}

impl From<ConnectError> for Error {
    fn from(err: ConnectError) -> Self {
        Self::Connect(err)
    }
}

impl From<DispatchError> for Error {
    fn from(err: DispatchError) -> Self {
        Self::Dispatch(err)
    }
}
