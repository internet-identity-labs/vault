export const idlFactory = ({ IDL }) => {
    const CreateResult = IDL.Record({ 'canister_id' : IDL.Principal });
    const Result = IDL.Variant({ 'Ok' : CreateResult, 'Err' : IDL.Text });
    return IDL.Service({
        'create_canister_call' : IDL.Func([], [Result], []),
    });
};
export const init = ({ IDL }) => { return []; };
