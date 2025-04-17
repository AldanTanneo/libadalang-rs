use std::ffi::c_void;

use libadalang_sys::{
    ada_analysis_context, ada_analysis_unit, ada_bool, ada_create_event_handler,
    ada_dec_ref_event_handler, ada_event_handler, ada_text,
};

use crate::{
    analysis::{Context, unit::Unit},
    exception::Exception,
    text::Text,
};

pub trait EventHandlerInterface: Sized + 'static {
    fn unit_requested(&mut self, ctx: Option<Context>, event: UnitRequestedEvent);
    fn unit_parsed(&mut self, ctx: Option<Context>, event: UnitParsedEvent);

    fn as_event_handler(self) -> Result<EventHandler, Exception> {
        EventHandler::new(self, Self::unit_requested, Self::unit_parsed)
    }
}

pub struct EventHandler(ada_event_handler);

struct EventHandlerData<D, F, G> {
    data: D,
    unit_requested_cb: F,
    unit_parsed_cb: G,
}

unsafe extern "C-unwind" fn destroy_cb<D, F, G>(data: *mut c_void) {
    let ptr = data.cast::<EventHandlerData<D, F, G>>();

    let boxed: Box<EventHandlerData<D, F, G>> = unsafe { Box::from_raw(ptr) };
    drop(boxed);
}

unsafe extern "C-unwind" fn unit_requested_cb<D, F, G>(
    data: *mut c_void,
    context: ada_analysis_context,
    name: *mut ada_text,
    from: ada_analysis_unit,
    found: ada_bool,
    is_not_found_error: ada_bool,
) where
    F: FnMut(&mut D, Option<Context>, UnitRequestedEvent) + 'static,
{
    let name = if name.is_null() {
        String::new()
    } else {
        String::from(Text::from_raw_borrow(&unsafe { core::ptr::read(name) }))
    };

    let from_unit = unsafe { Unit::from_raw(from) };
    let found = found != 0;
    let is_not_found_error = is_not_found_error != 0;

    let event = UnitRequestedEvent {
        name,
        from_unit,
        found,
        is_not_found_error,
    };

    let data = data.cast::<EventHandlerData<D, F, G>>();
    let data: &mut EventHandlerData<D, F, G> = unsafe { &mut *data };

    let ctx = unsafe { Context::from_raw(context) };

    (data.unit_requested_cb)(&mut data.data, ctx, event);
}

unsafe extern "C-unwind" fn unit_parsed_cb<D, F, G>(
    data: *mut c_void,
    context: ada_analysis_context,
    unit: ada_analysis_unit,
    reparsed: ada_bool,
) where
    G: FnMut(&mut D, Option<Context>, UnitParsedEvent) + 'static,
{
    let unit = unsafe { Unit::from_raw(unit) };
    let reparsed = reparsed != 0;

    let event = UnitParsedEvent { unit, reparsed };

    let data = data.cast::<EventHandlerData<D, F, G>>();
    let data: &mut EventHandlerData<D, F, G> = unsafe { &mut *data };

    let ctx = unsafe { Context::from_raw(context) };

    (data.unit_parsed_cb)(&mut data.data, ctx, event);
}

pub struct UnitRequestedEvent {
    pub name: String,
    pub from_unit: Unit,
    pub found: bool,
    pub is_not_found_error: bool,
}

pub struct UnitParsedEvent {
    pub unit: Unit,
    pub reparsed: bool,
}

impl EventHandler {
    fn new<D, F, G>(data: D, unit_requested: F, unit_parsed: G) -> Result<Self, Exception>
    where
        D: 'static,
        F: FnMut(&mut D, Option<Context>, UnitRequestedEvent) + 'static,
        G: FnMut(&mut D, Option<Context>, UnitParsedEvent) + 'static,
    {
        let boxed = Box::new(EventHandlerData {
            data,
            unit_requested_cb: unit_requested,
            unit_parsed_cb: unit_parsed,
        });

        let evh = unsafe {
            ada_create_event_handler(
                Box::into_raw(boxed).cast::<c_void>(),
                Some(destroy_cb::<D, F, G>),
                Some(unit_requested_cb::<D, F, G>),
                Some(unit_parsed_cb::<D, F, G>),
            )
        };

        Exception::wrap(EventHandler(evh))
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        unsafe { ada_dec_ref_event_handler(self.0) };
        Exception::log_and_ignore();
    }
}
