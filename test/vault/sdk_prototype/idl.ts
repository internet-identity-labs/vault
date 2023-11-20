export const idlFactory = ({ IDL }) => {
    const TransactionState = IDL.Variant({
        'Blocked' : IDL.Null,
        'Approved' : IDL.Null,
        'Rejected' : IDL.Null,
        'Executed' : IDL.Null,
        'Canceled' : IDL.Null,
        'Pending' : IDL.Null,
    });
    const TransactionApproveRequest = IDL.Record({
        'transaction_id' : IDL.Nat64,
        'state' : TransactionState,
    });
    const Network = IDL.Variant({
        'IC' : IDL.Null,
        'BTC' : IDL.Null,
        'ETH' : IDL.Null,
    });
    const TrType = IDL.Variant({
        'WalletUpdateName' : IDL.Null,
        'MemberCreate' : IDL.Null,
        'PolicyRemove' : IDL.Null,
        'MemberUnarchive' : IDL.Null,
        'WalletCreate' : IDL.Null,
        'MemberArchive' : IDL.Null,
        'PolicyCreate' : IDL.Null,
        'PolicyUpdate' : IDL.Null,
        'MemberUpdateName' : IDL.Null,
        'MemberUpdateRole' : IDL.Null,
        'QuorumUpdate' : IDL.Null,
        'Transfer' : IDL.Null,
    });
    const Approve = IDL.Record({
        'status' : TransactionState,
        'signer' : IDL.Text,
        'created_date' : IDL.Nat64,
    });
    const BasicTransactionFields = IDL.Record({
        'id' : IDL.Nat64,
        'transaction_type' : TrType,
        'initiator' : IDL.Text,
        'modified_date' : IDL.Nat64,
        'memo' : IDL.Opt(IDL.Text),
        'state' : TransactionState,
        'approves' : IDL.Vec(Approve),
        'is_vault_state' : IDL.Bool,
        'created_date' : IDL.Nat64,
        'batch_uid' : IDL.Opt(IDL.Text),
    });
    const WalletCreateTransaction = IDL.Record({
        'uid' : IDL.Text,
        'name' : IDL.Text,
        'network' : Network,
        'common' : BasicTransactionFields,
    });
    const Currency = IDL.Variant({ 'ICP' : IDL.Null });
    const PolicyCreateTransaction = IDL.Record({
        'uid' : IDL.Text,
        'member_threshold' : IDL.Nat8,
        'amount_threshold' : IDL.Nat64,
        'wallets' : IDL.Vec(IDL.Text),
        'currency' : Currency,
        'common' : BasicTransactionFields,
    });
    const VaultRole = IDL.Variant({ 'Member' : IDL.Null, 'Admin' : IDL.Null });
    const MemberUpdateRoleTransaction = IDL.Record({
        'id' : IDL.Text,
        'role' : VaultRole,
        'common' : BasicTransactionFields,
    });
    const PolicyRemoveTransaction = IDL.Record({
        'uid' : IDL.Text,
        'common' : BasicTransactionFields,
    });
    const WalletUpdateNameTransaction = IDL.Record({
        'uid' : IDL.Text,
        'name' : IDL.Text,
        'common' : BasicTransactionFields,
    });
    const PolicyUpdateTransaction = IDL.Record({
        'uid' : IDL.Text,
        'member_threshold' : IDL.Nat8,
        'amount_threshold' : IDL.Nat64,
        'common' : BasicTransactionFields,
    });
    const MemberArchiveTransaction = IDL.Record({
        'id' : IDL.Text,
        'common' : BasicTransactionFields,
    });
    const MemberCreateTransaction = IDL.Record({
        'id' : IDL.Text,
        'name' : IDL.Text,
        'role' : VaultRole,
        'common' : BasicTransactionFields,
    });
    const MemberUpdateNameTransaction = IDL.Record({
        'id' : IDL.Text,
        'name' : IDL.Text,
        'common' : BasicTransactionFields,
    });
    const QuorumUpdateTransaction = IDL.Record({
        'transaction_type' : TrType,
        'common' : BasicTransactionFields,
        'quorum' : IDL.Nat8,
    });
    const TransactionCandid = IDL.Variant({
        'WalletCreateTransactionV' : WalletCreateTransaction,
        'PolicyCreateTransactionV' : PolicyCreateTransaction,
        'MemberUpdateRoleTransactionV' : MemberUpdateRoleTransaction,
        'PolicyRemoveTransactionV' : PolicyRemoveTransaction,
        'WalletUpdateTransactionV' : WalletUpdateNameTransaction,
        'PolicyUpdateTransactionV' : PolicyUpdateTransaction,
        'MemberArchiveTransactionV' : MemberArchiveTransaction,
        'MemberUnarchiveTransactionV' : MemberArchiveTransaction,
        'MemberCreateTransactionV' : MemberCreateTransaction,
        'MemberUpdateNameTransactionV' : MemberUpdateNameTransaction,
        'QuorumUpdateTransactionV' : QuorumUpdateTransaction,
    });
    const ObjectState = IDL.Variant({
        'Active' : IDL.Null,
        'Archived' : IDL.Null,
    });
    const Member = IDL.Record({
        'id' : IDL.Text,
        'modified_date' : IDL.Nat64,
        'name' : IDL.Text,
        'role' : VaultRole,
        'state' : ObjectState,
        'created_date' : IDL.Nat64,
    });
    const Wallet = IDL.Record({
        'uid' : IDL.Text,
        'modified_date' : IDL.Nat64,
        'name' : IDL.Text,
        'network' : Network,
        'state' : ObjectState,
        'created_date' : IDL.Nat64,
    });
    const Quorum = IDL.Record({
        'modified_date' : IDL.Nat64,
        'quorum' : IDL.Nat8,
    });
    const Policy = IDL.Record({
        'uid' : IDL.Text,
        'member_threshold' : IDL.Nat8,
        'modified_date' : IDL.Nat64,
        'amount_threshold' : IDL.Nat64,
        'wallets' : IDL.Vec(IDL.Text),
        'currency' : Currency,
        'created_date' : IDL.Nat64,
    });
    const VaultState = IDL.Record({
        'members' : IDL.Vec(Member),
        'wallets' : IDL.Vec(Wallet),
        'quorum' : Quorum,
        'policies' : IDL.Vec(Policy),
    });
    const QuorumUpdateTransactionRequest = IDL.Record({ 'quorum' : IDL.Nat8 });
    const MemberUpdateNameTransactionRequest = IDL.Record({
        'member' : IDL.Text,
        'name' : IDL.Text,
    });
    const MemberArchiveTransactionRequest = IDL.Record({ 'member' : IDL.Text });
    const WalletCreateTransactionRequest = IDL.Record({
        'uid' : IDL.Text,
        'name' : IDL.Text,
        'network' : Network,
    });
    const MemberCreateTransactionRequest = IDL.Record({
        'member' : IDL.Text,
        'name' : IDL.Text,
        'role' : VaultRole,
    });
    const MemberUpdateRoleTransactionRequest = IDL.Record({
        'member' : IDL.Text,
        'role' : VaultRole,
    });
    const WalletUpdateNameTransactionRequest = IDL.Record({
        'uid' : IDL.Text,
        'name' : IDL.Text,
    });
    const TransactionRequest = IDL.Variant({
        'QuorumUpdateTransactionRequestV' : QuorumUpdateTransactionRequest,
        'MemberUpdateNameTransactionRequestV' : MemberUpdateNameTransactionRequest,
        'MemberUnArchiveTransactionRequestV' : MemberArchiveTransactionRequest,
        'WalletCreateTransactionRequestV' : WalletCreateTransactionRequest,
        'MemberCreateTransactionRequestV' : MemberCreateTransactionRequest,
        'MemberArchiveTransactionRequestV' : MemberArchiveTransactionRequest,
        'MemberUpdateRoleTransactionRequestV' : MemberUpdateRoleTransactionRequest,
        'WalletUpdateNameTransactionRequestV' : WalletUpdateNameTransactionRequest,
    });
    return IDL.Service({
        'approve' : IDL.Func(
            [IDL.Vec(TransactionApproveRequest)],
            [IDL.Vec(TransactionCandid)],
            [],
        ),
        'execute' : IDL.Func([], [], []),
        'get_state' : IDL.Func([IDL.Opt(IDL.Nat64)], [VaultState], ['query']),
        'get_transactions_all' : IDL.Func(
            [],
            [IDL.Vec(TransactionCandid)],
            ['query'],
        ),
        'request_transaction' : IDL.Func(
            [IDL.Vec(TransactionRequest)],
            [IDL.Vec(TransactionCandid)],
            [],
        ),
    });
};
export const init = ({ IDL }) => { return []; };
