var neon_dom = require('../native');

function create_closure(f, data) {
  return function callback(event) {
    f(data, event)
  }
}

neon_dom.init();
neon_dom.add_button(document, create_closure);

function handle_event(data, event) {
  console.log("Callback data is", data);
  console.log("Clicked at", event.screenX, event.screenY);
}

function all_the_buttons() {
  for (var i = 0; i < 100; i++) {
    var button = document.createElement("BUTTON");
    var button_text = document.createTextNode("click me");
    button.appendChild(button_text);
    document.body.appendChild(button);
    // button.onclick = function (event) {
    //   handle_event(42.0, event);
    // }
  }
}

function all_the_arrays() {
  var outer = [];
  for (var i = 0; i < 1000000; i++) {
    outer.push([]);
  }
  return outer;
}

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
  // console.time("rust arrays");
  // var outer = neon_dom.all_the_arrays();
  // console.timeEnd("rust arrays");
  // console.log(outer.length);

  // console.time("js arrays");
  // var outer = all_the_arrays();
  // console.timeEnd("js arrays");
  // console.log(outer.length);

  // document.body.innerHTML = "";

  // console.time("rust buttons");
  // neon_dom.all_the_buttons(document, create_closure);
  // console.timeEnd("rust buttons");

  // document.body.innerHTML = "";

  // console.time("js buttons");
  // all_the_buttons()
  // console.timeEnd("js buttons");

  // document.body.innerHTML = "";

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

window.setTimeout(function(event) {
  for (var i = 0; i < 10; i++) {
    bench();
  }
}, 1000);
