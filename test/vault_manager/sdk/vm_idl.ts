export const idlFactory = ({ IDL }) => {
    const CreateResult = IDL.Record({ 'canister_id' : IDL.Principal });
    const Result = IDL.Variant({ 'Ok' : CreateResult, 'Err' : IDL.Text });
    const VaultCanister = IDL.Record({
        'initiator' : IDL.Principal,
        'canister_id' : IDL.Principal,
        'block_number' : IDL.Nat64,
    });
    const Conf = IDL.Record({
        'destination_address' : IDL.Text,
        'initial_cycles_balance' : IDL.Nat,
        'origins' : IDL.Vec(IDL.Text),
        'repo_canister_id' : IDL.Text,
        'icp_price' : IDL.Nat64,
    });
    const Result_1 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
    return IDL.Service({
        'canister_balance' : IDL.Func([], [IDL.Nat64], ['query']),
        'create_canister_call' : IDL.Func([IDL.Nat64], [Result], []),
        'get_all_canisters' : IDL.Func([], [IDL.Vec(VaultCanister)], ['query']),
        'get_config' : IDL.Func([], [Conf], ['query']),
        'update_canister_self' : IDL.Func([IDL.Text], [Result_1], []),
    });
};
export const init = ({ IDL }) => { return []; };
