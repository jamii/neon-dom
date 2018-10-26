#![feature(nll)]

#[macro_use]
extern crate neon;

use neon::prelude::*;

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
    let screen_x = event
        .get(&mut cx, "screenX")
        .unwrap()
        .downcast::<JsNumber>()
        .unwrap()
        .value();
    let screen_y = event
        .get(&mut cx, "screenY")
        .unwrap()
        .downcast::<JsNumber>()
        .unwrap()
        .value();

    println!("Clicked at {} {}", screen_x, screen_y);

    Ok(cx.null())
}

fn add_button(mut cx: FunctionContext) -> JsResult<JsNull> {
    assert!(cx.len() == 2);

    let document = cx.argument::<JsObject>(0).unwrap();
    let create_callback = cx.argument::<JsFunction>(1).unwrap();

    // var button = document.createElement("BUTTON")
    let create_element = document
        .get(&mut cx, "createElement")
        .unwrap()
        .downcast::<JsFunction>()
        .unwrap();
    let button_string = cx.string("BUTTON");
    let button = create_element
        .call(&mut cx, document, vec![button_string])
        .unwrap()
        .downcast::<JsObject>()
        .unwrap();

    // var button_text = document.createTextNode("click me")
    let create_text_node = document
        .get(&mut cx, "createTextNode")
        .unwrap()
        .downcast::<JsFunction>()
        .unwrap();
    let click_me_string = cx.string("click me");
    let button_text = create_text_node
        .call(&mut cx, document, vec![click_me_string])
        .unwrap();

    // button.appendChild(button_text)
    let append_child = button
        .get(&mut cx, "appendChild")
        .unwrap()
        .downcast::<JsFunction>()
        .unwrap();
    append_child
        .call(&mut cx, button, vec![button_text])
        .unwrap();

    // body.appendChild(button)
    let body = document
        .get(&mut cx, "body")
        .unwrap()
        .downcast::<JsObject>()
        .unwrap();
    let append_child = body
        .get(&mut cx, "appendChild")
        .unwrap()
        .downcast::<JsFunction>()
        .unwrap();
    append_child.call(&mut cx, body, vec![button]).unwrap();

    // button.onclick = function (event) {handle_event(42, event);}
    let this = cx.null();
    let callback_data = cx.number(42);
    let callback_function = JsFunction::new(&mut cx, handle_event).unwrap();
    let callback = create_callback
        .call(
            &mut cx,
            this,
            vec![
                callback_function.upcast::<JsValue>(),
                callback_data.upcast::<JsValue>(),
            ],
        )
        .unwrap();
    button.set(&mut cx, "onclick", callback).unwrap();

    Ok(cx.null())
}

register_module!(mut cx, {
    cx.export_function("init", init)?;
    cx.export_function("add_button", add_button)?;
    Ok(())
});
