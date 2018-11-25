#![feature(trace_macros)]
#![feature(nll)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate neon;
extern crate neon_serde;
extern crate rand;
extern crate rusqlite;

use neon::prelude::*;
use rand::{Rng, SeedableRng};
use std::sync::Once;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::sleep;
use std::time::Duration;

#[macro_use]
mod sugar;

use sugar::*;

// --- WORKER ---
// will be run in a background thread

fn run_query(query: &str) -> String {
    // let's make the wait noticeable
    sleep(Duration::from_millis(1000));

    let db = rusqlite::Connection::open_with_flags(
        "./chinook.db",
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();

    let mut statement = db.prepare(&*query).unwrap();
    let rows = statement
        .query_map(rusqlite::NO_PARAMS, |row| {
            (0..row.column_count())
                .into_iter()
                .map(|i| row.get::<usize, String>(i))
                .collect::<Vec<_>>()
                .join("\t")
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    rows.join("\n")
}

// --- MODEL ---
// state, event handling and rendering

type Id = usize;

enum Answer {
    Pending,
    Answered(String),
}

struct Model {
    query: String,
    next_id: Id,
    answers: Vec<(Id, Answer)>,
}

#[derive(Serialize, Deserialize)]
enum Event {
    RunQuery,
}

impl Model {
    fn new() -> Self {
        Model {
            query: "".to_owned(),
            next_id: 0,
            answers: vec![],
        }
    }

    fn handle_event<SW>(&mut self, event: Event, spawn_worker: SW)
    where
        SW: Fn(Box<dyn FnMut(&mut Model)>),
    {
        match event {
            Event::RunQuery => {
                let id = self.next_id;
                self.next_id += 1;
                let query = ::std::mem::replace(&mut self.query, "".to_owned());
                self.answers.push((id, Answer::Pending));
                spawn_worker(Box::new(move |model| {
                    let answer = run_query(&*query);
                    for (id2, answer2) in model.answers.iter_mut() {
                        if id == *id2 {
                            *answer2 = Answer::Answered(answer);
                        }
                        break;
                    }
                }));
            }
        }
    }

    fn render<'a, CX>(
        &self,
        cx: &mut CX,
        document: Handle<JsValue>,
        create_closure: Handle<JsValue>,
    ) where
        CX: Context<'a>,
    {
        js!(cx, document.body.innerHTML = "");

        let wrapper = js!(cx, document.createElement("div"));
        js!(cx, document.body.appendChild(&wrapper));

        let hello = js!(cx, document.createTextNode("hello"));
        js!(cx, wrapper.appendChild(&hello));
    }
}

// --- APP ---
// coordinates model updates and rendering
// may even be correct

pub struct App {
    model: Mutex<Model>,
    needs_render: (Mutex<bool>, Condvar),
}

impl App {
    fn new() -> Self {
        App {
            model: Mutex::new(Model::new()),
            needs_render: (Mutex::new(true), Condvar::new()),
        }
    }

    fn handle_event(&self, event: Event) {
        let mut model_guard = self.model.lock().unwrap();
        model_guard.handle_event(event, |mut worker: Box<dyn FnMut(&mut Model)>| {
            let mut model_guard = self.model.lock().unwrap();
            worker(&mut *model_guard);
            self.set_needs_render();
            drop(model_guard);
        });
        self.set_needs_render();
        drop(model_guard);
    }

    fn set_needs_render(&self) {
        let (mutex, condvar) = &self.needs_render;
        let mut guard = mutex.lock().unwrap();
        *guard = true;
        condvar.notify_all();
        drop(guard);
    }

    fn wait_until_needs_render(&self) {
        let (mutex, condvar) = &self.needs_render;
        let mut guard = mutex.lock().unwrap();
        while !*guard {
            guard = condvar.wait(guard).unwrap();
        }
        drop(guard);
    }

    fn render<'a, CX>(
        &self,
        cx: &mut CX,
        document: Handle<JsValue>,
        create_closure: Handle<JsValue>,
    ) where
        CX: Context<'a>,
    {
        let model_guard = self.model.lock().unwrap();
        model_guard.render(cx, document, create_closure);
        self.set_needs_render();
        drop(model_guard);
    }
}

// --- ELECTRON BOILERPLATE ---

struct OnNeedsRender {
    app: Arc<App>,
}

impl neon::task::Task for OnNeedsRender {
    type Output = ();
    type Error = ();
    type JsEvent = JsNull;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        self.app.wait_until_needs_render();
        Ok(())
    }

    fn complete<'a>(
        self,
        mut cx: TaskContext<'a>,
        result: Result<Self::Output, Self::Error>,
    ) -> JsResult<Self::JsEvent> {
        result.unwrap();
        Ok(cx.null())
    }
}

// the declare_types macro doesn't like Arc<App>
type ArcApp = Arc<App>;

declare_types! {

    pub class JsApp for ArcApp {
        init(mut _cx) {
            Ok(Arc::new(App::new()))
        }

        method handle_event(mut cx) {
            assert!(cx.len() == 1);
            // TODO

            let this = cx.this();
            let app: Arc<App> = {
                let guard = cx.lock();
                let borrow = this.borrow(&guard);
                borrow.clone()
            };

            // TODO

            Ok(cx.null().upcast())
        }

        method on_needs_render(mut cx) {
            assert!(cx.len() == 1);
            let callback = cx.argument::<JsFunction>(0).unwrap();

            let this = cx.this();
            let app: Arc<App> = {
                let guard = cx.lock();
                let borrow = this.borrow(&guard);
                borrow.clone()
            };

            OnNeedsRender {
                app: app
            }.schedule(callback);

            Ok(cx.null().upcast())
        }

        method render(mut cx) {
            assert!(cx.len() == 2);
            let document = cx.argument::<JsValue>(0).unwrap();
            let create_closure = cx.argument::<JsValue>(1).unwrap();

            let this = cx.this();
            let app: Arc<App> = {
                let guard = cx.lock();
                let borrow = this.borrow(&guard);
                borrow.clone()
            };

            app.render(&mut cx, document, create_closure);

            Ok(cx.null().upcast())
        }
    }
}

// --- MICROBENCHMARKS ---

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
        Node::Text(text) => js!(cx, document.createTextNode(text)),
        Node::Div(nodes) => {
            let parent_element = js!(cx, document.createElement("div"));
            for child_node in nodes.iter() {
                let child_element = make_node(cx, document, child_node);
                js!(cx, parent_element.appendChild(child_element));
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
    js!(&mut cx, document.body.appendChild(node_element));
    Ok(cx.null())
}

// --- EXPORTED TO ELECTRON ---

static INIT: Once = Once::new();

register_module!(mut cx, {
    INIT.call_once(|| {
        // this is the easiest way to get backtraces out of electron
        simple_logger::init().unwrap();
        log_panics::init();
    });

    cx.export_class::<JsApp>("App")?;
    cx.export_function("get_the_node", get_the_node)?;
    cx.export_function("put_the_node", put_the_node)?;

    Ok(())
});
