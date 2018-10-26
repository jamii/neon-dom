var neon_dom = require('../native');

function create_callback(f, data) {
  console.log("Making callback", f, data);
  return function callback(event) {
    console.log("Calling callback", f, data, event);
    f(data, event)
  }
}

neon_dom.init();
neon_dom.add_button(document, create_callback);
