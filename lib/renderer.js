var neon_dom = require('../native');

function create_callback(f, data) {
  return function callback(event) {
    f(data, event)
  }
}

neon_dom.init();
neon_dom.add_button(document, create_callback);
