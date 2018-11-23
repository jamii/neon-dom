#![feature(nll)]
#![feature(trace_macros)]
#![feature(specialization)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate neon;
#[macro_use]
extern crate neon_serde;
extern crate rand;

use neon::prelude::*;
use rand::{Rng, SeedableRng};

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

impl ToJs for str {
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
    // ( $cx:expr, arguments $( $rest:tt )+ ) => {{
    //     js_!(@chain,
    //          $cx,
    //          $value,
    //          $( $rest )*)
    // }};
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

fn all_the_buttons(mut cx: FunctionContext) -> JsResult<JsNull> {
    assert!(cx.len() == 2);
    let document = cx.argument::<JsValue>(0).unwrap();
    let create_closure = cx.argument::<JsValue>(1).unwrap();

    for _ in 0..1_00 {
        let button = js_!(&mut cx, document.createElement("BUTTON"));
        let button_text = js_!(&mut cx, document.createTextNode("click me"));
        js_!(&mut cx, button.appendChild(button_text));
        js_!(&mut cx, document.body.appendChild(button));

        // let callback_function = JsFunction::new(&mut cx, handle_event)
        //     .unwrap()
        //     .upcast::<JsValue>();
        // js_!(
        //     &mut cx,
        //     button.onclick = create_closure(callback_function, 42.0)
        // );
    }

    Ok(cx.null())
}

fn all_the_arrays(mut cx: FunctionContext) -> JsResult<JsArray> {
    let outer = cx.empty_array();

    for _ in 0..1_000_000 {
        let inner = cx.empty_array();
        let len = outer.len();
        outer.set(&mut cx, len, inner)?;
    }

    Ok(outer)
}

#[derive(Serialize)]
enum Node {
    Text(String),
    Div(Vec<Node>),
}

fn random_node() -> Node {
    let mut rng = rand::StdRng::from_seed(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    let mut nodes = vec![];
    for _ in 0..1_000_0 {
        if rng.gen() {
            nodes.push(Node::Text(rng.gen::<usize>().to_string()))
        } else {
            nodes = vec![Node::Div(nodes)];
        }
    }
    Node::Div(nodes)
}

fn make_node<'a>(
    cx: &mut FunctionContext<'a>,
    document: Handle<'a, JsValue>,
    node: &Node,
) -> Handle<'a, JsValue> {
    match node {
        Node::Text(text) => js_!(cx, document.createTextNode(text)),
        Node::Div(nodes) => {
            let parent_element = js_!(cx, document.createElement("div"));
            for child_node in nodes.iter() {
                let child_element = make_node(cx, document, child_node);
                js_!(cx, parent_element.appendChild(child_element));
            }
            parent_element
        }
    }
}

fn get_the_node(mut cx: FunctionContext) -> JsResult<JsValue> {
    Ok(neon_serde::to_value(&mut cx, &random_node()).unwrap())
}

fn put_the_node(mut cx: FunctionContext) -> JsResult<JsNull> {
    assert!(cx.len() == 1);
    let document = cx.argument::<JsValue>(0).unwrap();
    let node_element = make_node(&mut cx, document, &random_node());
    js_!(&mut cx, document.body.appendChild(node_element));
    Ok(cx.null())
}

register_module!(mut cx, {
    cx.export_function("init", init)?;
    cx.export_function("add_button", add_button)?;
    cx.export_function("all_the_buttons", all_the_buttons)?;
    cx.export_function("all_the_arrays", all_the_arrays)?;
    cx.export_function("get_the_node", get_the_node)?;
    cx.export_function("put_the_node", put_the_node)?;
    Ok(())
});
