use tracing::{
    Event, Metadata, Subscriber,
    field::{Field, Visit},
    level_filters::LevelFilter,
    span::{Attributes, Id, Record},
};

// Only static tracing messages are emitted. Fields may contain sensitive data.
struct Console(LevelFilter);
struct Message;
impl Visit for Message {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            eprintln!("{value:?}");
        }
    }
}
impl Subscriber for Console {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= &self.0
            && (metadata.target().starts_with("nemu_core::")
                || metadata.target().starts_with("core_store::"))
    }
    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(1)
    }
    fn record(&self, _: &Id, _: &Record<'_>) {}
    fn record_follows_from(&self, _: &Id, _: &Id) {}
    fn event(&self, event: &Event<'_>) {
        event.record(&mut Message);
    }
    fn enter(&self, _: &Id) {}
    fn exit(&self, _: &Id) {}
}

pub fn install(level: LevelFilter) -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    tracing::subscriber::set_global_default(Console(level))
}
