var neon_dom = require('../native');
var app = new neon_dom.App()

function app_loop() {
  app.on_needs_render(function () {
    window.requestAnimationFrame(function () {
      app.render(document,
                 function create_handler(event) {
                   return function handler(dom_event) {
                     app.handle_event(event, dom_event);

                     console.log("Handler returned control");
                   }
                 });
      console.log("Rendering returned control");
      app_loop();
    });
  });
}

app_loop();

// --- MICROBENCHMARKS ---

function make_node(node) {
  if (node.Text)  {
    return document.createTextNode(node.Text);
  } else {
    var nodes = node.Div;
    var parent_element = document.createElement("div");
    for (var i = 0; i < nodes.length; i++) {
      var child_element = make_node(nodes[i]);
      parent_element.appendChild(child_element);
    }
    return parent_element;
  }
}

function bench() {

  document.body.innerHTML = "";

  console.time("rust nodes");
  neon_dom.put_the_node(document);
  console.timeEnd("rust nodes");

  document.body.innerHTML = "";

  console.time("js nodes");
  console.time("js nodes (get)")
  var node = neon_dom.get_the_node();
  console.timeEnd("js nodes (get)")
  console.time("js nodes (make)")
  var node_element = make_node(node);
  console.timeEnd("js nodes (make)")
  document.body.appendChild(node_element);
  console.timeEnd("js nodes");

  document.body.innerHTML = "";
}

// // highly scientific
// for (var i = 0; i < 10; i++) {
//     bench();
//   }
