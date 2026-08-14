# Re-entrancy

## Soroban's re-entrancy model

Soroban does not allow a contract to re-enter itself within a single call stack.
If contract A calls contract B, and contract B attempts to call back into contract A,
the host returns an error. This is enforced at the host level, not by application code.

Reference: Stellar documentation on Soroban contract invocation semantics.

## Does this contract have a re-entrancy risk?

No. This contract does not call any external contract at all. There are no
`env.invoke_contract` calls in `contract.rs`. The only external interactions are:

1. `address.require_auth()` — a host built-in, not a cross-contract call.
2. `env.events().publish(...)` — a host built-in, not a cross-contract call.
3. `env.storage().*` — host storage, not a cross-contract call.
4. `env.ledger().timestamp()` — host ledger access, not a cross-contract call.

Because this contract never transfers control to another contract, the re-entrancy
attack vector does not apply here.

## Relevance if this contract is wrapped

If another contract wraps this one (e.g. a payment coordinator that calls
`pay_invoice` and then calls a token contract), re-entrancy considerations apply
to the *wrapper*, not to this contract. The Soroban host's cross-contract
re-entrancy protection would still apply, but the wrapper author is responsible
for ensuring correct call ordering.
