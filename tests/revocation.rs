//! Revocation cases for p2panda-auth 0.7.1 under its default "strong removal" resolver.
//!
//! Each expected member list comes from the crate's documented rules (lib.rs, "Concurrency
//! Resolution"), not from running the code: (1) removing a manager invalidates that member's
//! concurrent actions; (2) mutual concurrent removals both apply; (3) a removed and re-added
//! member stays a member but loses its concurrent actions. Every case is checked in every
//! delivery order that respects the operations' dependencies, because peers receive group
//! operations in different orders and must still agree.

use p2panda_auth::group::GroupMember;
use p2panda_auth::test_utils::{
    add_member, create_group, remove_member, MemberId, TestGroup, TestGroupState, TestOperation,
};
use p2panda_auth::traits::Operation;
use p2panda_auth::Access;

const G: MemberId = 'G';
const ALICE: MemberId = 'A';
const BOB: MemberId = 'B';
const CLAIRE: MemberId = 'C';
const DAVE: MemberId = 'D';

/// Every order of `ops` in which each operation comes after the operations it depends on.
fn causal_orders(ops: &[TestOperation]) -> Vec<Vec<TestOperation>> {
    fn walk(left: &[TestOperation], done: &mut Vec<TestOperation>, out: &mut Vec<Vec<TestOperation>>) {
        if left.is_empty() {
            out.push(done.clone());
            return;
        }
        for (i, op) in left.iter().enumerate() {
            let ready = op.dependencies().iter().all(|d| done.iter().any(|x| x.id() == *d));
            if ready {
                let mut rest = left.to_vec();
                rest.remove(i);
                done.push(op.clone());
                walk(&rest, done, out);
                done.pop();
            }
        }
    }
    let mut out = Vec::new();
    walk(ops, &mut Vec::new(), &mut out);
    out
}

fn members_after(order: &[TestOperation]) -> Vec<(MemberId, Access<()>)> {
    let state = order.iter().fold(TestGroupState::new(), |state, op| {
        TestGroup::process(state, op).expect("operation accepted")
    });
    let mut members = state.members(G);
    members.sort();
    members
}

fn assert_every_order(ops: &[TestOperation], expected: &[(MemberId, Access<()>)]) -> usize {
    let orders = causal_orders(ops);
    let mut expected = expected.to_vec();
    expected.sort();
    for order in &orders {
        let ids: Vec<u32> = order.iter().map(|op| op.id()).collect();
        assert_eq!(members_after(order), expected, "delivery order {ids:?}");
    }
    orders.len()
}

fn individual(id: MemberId) -> GroupMember<MemberId> {
    GroupMember::Individual(id)
}

#[test]
fn removing_a_manager_drops_the_add_it_made_concurrently() {
    // Alice creates the group and makes Bob a manager. Concurrently: Alice removes Bob, and
    // Bob adds Claire. Rule 1: Bob's add is invalidated, so Claire is not a member.
    let create = create_group(ALICE, 0, G, vec![(individual(ALICE), Access::manage())], vec![]);
    let add_bob = add_member(ALICE, 1, G, individual(BOB), Access::manage(), vec![0]);
    let bob_adds_claire = add_member(BOB, 2, G, individual(CLAIRE), Access::write(), vec![1]);
    let alice_removes_bob = remove_member(ALICE, 3, G, individual(BOB), vec![1]);
    let n = assert_every_order(
        &[create, add_bob, bob_adds_claire, alice_removes_bob],
        &[(ALICE, Access::manage())],
    );
    assert_eq!(n, 2);
}

#[test]
fn mutual_removal_applies_both_removals() {
    // Alice and Bob, both managers, remove each other concurrently; Claire was added first.
    // Rule 2: both removals apply, so only Claire is left.
    let create = create_group(ALICE, 0, G, vec![(individual(ALICE), Access::manage())], vec![]);
    let add_bob = add_member(ALICE, 1, G, individual(BOB), Access::manage(), vec![0]);
    let add_claire = add_member(ALICE, 2, G, individual(CLAIRE), Access::read(), vec![1]);
    let alice_removes_bob = remove_member(ALICE, 3, G, individual(BOB), vec![2]);
    let bob_removes_alice = remove_member(BOB, 4, G, individual(ALICE), vec![2]);
    let n = assert_every_order(
        &[create, add_bob, add_claire, alice_removes_bob, bob_removes_alice],
        &[(CLAIRE, Access::read())],
    );
    assert_eq!(n, 2);
}

#[test]
fn a_re_added_manager_stays_but_loses_its_concurrent_add() {
    // Alice removes Bob and then re-adds him. Concurrently with the removal, Bob adds Dave.
    // Rule 3: Bob is a member again, but his concurrent add is invalidated.
    let create = create_group(ALICE, 0, G, vec![(individual(ALICE), Access::manage())], vec![]);
    let add_bob = add_member(ALICE, 1, G, individual(BOB), Access::manage(), vec![0]);
    let bob_adds_dave = add_member(BOB, 2, G, individual(DAVE), Access::write(), vec![1]);
    let alice_removes_bob = remove_member(ALICE, 3, G, individual(BOB), vec![1]);
    let alice_re_adds_bob = add_member(ALICE, 4, G, individual(BOB), Access::manage(), vec![3]);
    let n = assert_every_order(
        &[create, add_bob, bob_adds_dave, alice_removes_bob, alice_re_adds_bob],
        &[(ALICE, Access::manage()), (BOB, Access::manage())],
    );
    assert_eq!(n, 3);
}
