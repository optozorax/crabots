var ctx = null;
var memory;

params_set_mem = function (wasm_memory, _wasm_exports) {
    memory = wasm_memory;
    ctx = {};
    ctx.entries = [];
    var some = new URLSearchParams(window.location.search);
    for (i of some.entries()) {
        ctx.entries.push(i);
    }
}

params_register_js_plugin = function (importObject) {
    importObject.env.param_count = function () {
        return ctx.entries.length;
    }
    importObject.env.param_key_length = function (i) {
        return ctx.entries[i][0].length;
    }
    importObject.env.param_key_letter = function (i, j) {
        return ctx.entries[i][0][j].charCodeAt(0);
    }
    importObject.env.param_value_length = function (i) {
        return ctx.entries[i][1].length;
    }
    importObject.env.param_value_letter = function (i, j) {
        return ctx.entries[i][1][j].charCodeAt(0);
    }
}

miniquad_add_plugin({
    register_plugin: params_register_js_plugin,
    on_init: params_set_mem
});
