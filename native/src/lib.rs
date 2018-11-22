#![feature(nll)]
#![feature(trace_macros)]
#![feature(specialization)]

extern crate serde;
#[macro_use]
extern crate neon;
#[macro_use]
extern crate neon_serde;

use neon::prelude::*;

// trace_macros!(true);

trait ToJs {
    fn to_js<'a>(&self, cx: &mut FunctionContext<'a>) -> Handle<'a, JsValue>
    where
        Self: 'a;
}

impl ToJs for f64 {
    fn to_js<'a>(&self, cx: &mut FunctionContext<'a>) -> Handle<'a, JsValue>
    where
        Self: 'a,
    {
        neon_serde::to_value(cx, self).unwrap()
    }
}

impl<'b> ToJs for &'b str {
    fn to_js<'a>(&self, cx: &mut FunctionContext<'a>) -> Handle<'a, JsValue>
    where
        Self: 'a,
    {
        neon_serde::to_value(cx, self).unwrap()
    }
}

impl<'b> ToJs for Handle<'b, JsValue> {
    fn to_js<'a>(&self, _cx: &mut FunctionContext<'a>) -> Handle<'a, JsValue>
    where
        Self: 'a,
    {
        self.clone()
    }
}

macro_rules! js_ {
    ( @chain, $cx:expr, $value:expr , ) => {{
        $value
    }};
    ( @chain, $cx:expr, $value:expr , . $key:ident = $( $rest:tt )* ) => {{
        let value = $value;
        let rest = js_!($cx, $( $rest )*);
        value
            .downcast::<JsObject>()
            .unwrap()
            .set($cx, stringify!($key), rest)
            .unwrap()
    }};
    ( @chain, $cx:expr, $value:expr , . $key:ident ( $( $args:expr ),* ) $( $rest:tt )* ) => {{
        js_!(@chain,
             $cx,
             {
                 let value = $value;
                 let function = value
                     .downcast::<JsObject>()
                     .unwrap()
                     .get($cx, stringify!($key))
                     .unwrap();
                 let mut args = vec![];
                 {
                     $( args.push(js_!($cx, $args)); )*
                 }
                 function
                     .downcast::<JsFunction>()
                     .unwrap()
                     .call($cx, value, args)
                     .unwrap()
             },
             $( $rest )*)
    }};
    ( @chain, $cx:expr, $value:expr , . $key:ident $( $rest:tt )* ) => {{
        let value = $value;
        js_!(@chain,
             $cx,
             value
             .downcast::<JsObject>()
             .unwrap()
             .get($cx, stringify!($key))
             .unwrap(),
             $( $rest )*)
    }};
    ( @chain, $cx:expr, $value:expr , ( $( $args:expr ),* ) $( $rest:tt )* ) => {{
        js_!(@chain,
             $cx,
             {
                 let value = $value;
                 let mut args = vec![];
                 {
                     $( args.push(js_!($cx, $args)); )*
                 }
                 let null = ($cx).null();
                 value
                     .downcast::<JsFunction>()
                     .unwrap()
                     .call($cx, null, args)
                     .unwrap()
             },
             $( $rest )*)
    }};
    ( $cx:expr, $value:ident $( $rest:tt )+ ) => {{
        js_!(@chain,
             $cx,
             $value,
             $( $rest )*)
    }};
    ( $cx:expr, null ) => {{
        ($cx).null()
    }};
    ( $cx:expr, $expr:expr ) => {{
        ($expr).to_js($cx)
    }}
}

macro_rules! js {
    ( $cx:expr, $( $args:tt )* ) => {{
        let value = js_!( $cx, $( $args )* );
        neon_serde::from_value($cx, value).unwrap()
    }};
}

// this is the easiest way to get backtraces out of electron
fn init(mut cx: FunctionContext) -> JsResult<JsNull> {
    simple_logger::init().unwrap();
    log_panics::init();
    Ok(cx.null())
}

fn handle_event(mut cx: FunctionContext) -> JsResult<JsNull> {
    assert!(cx.len() == 2);
    let data = cx.argument::<JsNumber>(0).unwrap().value();
    println!("Callback data is {}", data);

    let event = cx.argument::<JsObject>(1).unwrap();
    let screen_x: f64 = js!(&mut cx, event.screenX);
    let screen_y: f64 = js!(&mut cx, event.screenY);
    println!("Clicked at {} {}", screen_x, screen_y);

    Ok(cx.null())
}

fn add_button(mut cx: FunctionContext) -> JsResult<JsNull> {
    assert!(cx.len() == 2);
    let document = cx.argument::<JsValue>(0).unwrap();
    let create_closure = cx.argument::<JsValue>(1).unwrap();

    let button = js_!(&mut cx, document.createElement("BUTTON"));
    let button_text = js_!(&mut cx, document.createTextNode("click me"));
    js_!(&mut cx, button.appendChild(button_text));
    js_!(&mut cx, document.body.appendChild(button));

    let callback_function = JsFunction::new(&mut cx, handle_event)
        .unwrap()
        .upcast::<JsValue>();
    js_!(
        &mut cx,
        button.onclick = create_closure(callback_function, 42.0)
    );

    Ok(cx.null())
}

register_module!(mut cx, {
    cx.export_function("init", init)?;
    cx.export_function("add_button", add_button)?;
    Ok(())
});
