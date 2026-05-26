use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_logging(trace: bool, debug: bool, verbose: bool) {
    let level = if trace {
        Level::TRACE
    } else if debug {
        Level::DEBUG
    } else if verbose {
        Level::INFO
    } else {
        Level::WARN
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();

    subscriber.init();
}
