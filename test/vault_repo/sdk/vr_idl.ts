export const idlFactory = ({ IDL }) => {
    const VaultWasm = IDL.Record({
        'wasm_module' : IDL.Vec(IDL.Nat8),
        'hash' : IDL.Text,
        'description' : IDL.Opt(IDL.Text),
        'version' : IDL.Text,
    });
    return IDL.Service({
        'add_version' : IDL.Func([VaultWasm], [], []),
        'canister_balance' : IDL.Func([], [IDL.Nat64], ['query']),
        'get_available_versions' : IDL.Func([], [IDL.Vec(IDL.Text)], []),
        'get_by_version' : IDL.Func([IDL.Text], [VaultWasm], ['query']),
        'get_latest_version' : IDL.Func([], [VaultWasm], ['query']),
        'sync_controllers' : IDL.Func([], [IDL.Vec(IDL.Text)], []),
        'get_trusted_origins' : IDL.Func([], [IDL.Vec(IDL.Text)], []),
    });
};
export const init = ({ IDL }) => { return []; };
