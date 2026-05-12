// lib/norick.js — No-Rick ZK proof client. Client-side halo2 proof generation via WASM.

let wasm = null;
let wasmReady = false;

/**
 * Load the norick-wasm module (built by wasm-pack).
 * Falls back gracefully if WASM is not available.
 */
export async function loadWasm() {
    if (wasmReady) return wasm;
    try {
        const mod = await import('/pkg/norick_wasm.js');
        await mod.default();
        wasm = mod;
        wasmReady = true;
        return wasm;
    } catch (e) {
        console.warn('norick-wasm not available', e);
        return null;
    }
}

export function isWasmReady() { return wasmReady; }

/**
 * Generate a halo2 proof that `secretWord` does not contain `forbidden`.
 * Runs the full circuit + prover client-side in WASM.
 * @param {string} secretWord — the user's secret (max 20 chars)
 * @param {string} forbidden — the forbidden word (e.g. "rick")
 * @returns {string} base64-encoded proof bytes
 */
export function generateProof(secretWord, forbidden) {
    if (!wasmReady) throw new Error('WASM not loaded');
    return wasm.generate_proof(secretWord, forbidden);
}

/**
 * Encode circuit instances for the forbidden word.
 * Returns Uint8Array of serialized field elements.
 */
export function encodeInstances(forbidden) {
    if (!wasmReady) throw new Error('WASM not loaded');
    return wasm.encode_instances(forbidden);
}

/**
 * Build the JSON ExecuteMsg for the zk-wasmvm-test contract.
 * @param {number} cid — circuit ID (code_id)
 * @param {string} forbidden — the forbidden word
 * @param {string} proofB64 — base64-encoded proof bytes
 * @returns {string} JSON string of the execute message
 */
export function encodeProofMsg(cid, forbidden, proofB64) {
    if (!wasmReady) throw new Error('WASM not loaded');
    return wasm.encode_proof_msg(BigInt(cid), forbidden, proofB64);
}

/**
 * Get hex representation of a string's field element (debugging).
 */
export function strToFieldHex(s) {
    if (!wasmReady) throw new Error('WASM not loaded');
    return wasm.str_to_field_hex(s);
}
