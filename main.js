import init, { run_app } from './pkg/path_finding_playground_wasm.js';
async function main() {
    await init('/pkg/path_finding_playground_wasm_bg.wasm');
    run_app();
}
main()
