export const idlFactory = ({ IDL }) => {
    const CreateResult = IDL.Record({ 'canister_id' : IDL.Principal });
    const Result = IDL.Variant({ 'Ok' : CreateResult, 'Err' : IDL.Text });
    const VaultCanister = IDL.Record({
        'initiator' : IDL.Principal,
        'canister_id' : IDL.Principal,
    });
    return IDL.Service({
        'canister_balance' : IDL.Func([], [IDL.Nat64], ['query']),
        'create_canister_call' : IDL.Func([IDL.Nat64], [Result], []),
        'get_all_canisters' : IDL.Func([], [IDL.Vec(VaultCanister)], ['query']),
    });
};
export const init = ({ IDL }) => { return []; };
