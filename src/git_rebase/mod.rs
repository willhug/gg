use crate::git::{parse_branch, ParsedBranch, get_commit_hash, checkout, reset, delete_branch_all, new, cherry_pick, current_branch, get_branch_for_dir, current_parsed_branch, cherry_abort, cherry_continue, assert_branch_exists, get_children_branches, delete_branch_local};

pub(crate) fn rebase_all_children(strategy: Option<String>) {
    let cur = current_parsed_branch();
    let children = get_children_branches(&cur);

    for child in children {
        checkout(&child.full());
        start_rebase(None, strategy.clone());
    }
}

pub(crate) fn start_rebase(onto: Option<String>, strategy: Option<String>) {
    let cur = TmpBranchWrapper::new(current_parsed_branch());
    let onto = onto.unwrap_or_else(|| {
        get_branch_for_dir(crate::git::CheckoutDir::Prev).expect("No previous branch to rebase onto")
    });
    assert_branch_exists(onto.clone());
    rebase_onto(cur, onto, strategy);
}

pub(crate) fn abort_rebase() {
    let br = TmpBranchWrapper::new_from_tmp_branch(current_branch());

    cherry_abort();

    checkout(&br.inner.full());

    delete_branch_all(br.tmp_branch_name());
    delete_branch_all(br.tmp_start_branch_name());
}

pub(crate) fn continue_rebase() {
    let br = TmpBranchWrapper::new_from_tmp_branch(current_branch());

    cherry_continue();

    finish_rebase(br);
}

fn rebase_onto(branch_to_rebase: TmpBranchWrapper, onto: String, strategy: Option<String>) {
    println!("Rebasing {} onto {} via cherry-picks", branch_to_rebase.inner.full(), onto);
    checkout(&onto);
    new(branch_to_rebase.tmp_start_branch_name().as_str());
    new(branch_to_rebase.tmp_branch_name().as_str());

    if branches_are_equivalent(branch_to_rebase.inner.start(), branch_to_rebase.inner.full()) {
        println!("There are no commits to rebase, fast forwarding the branches");
        checkout(&branch_to_rebase.inner.start());
        reset(branch_to_rebase.tmp_branch_name(), true);
        checkout(&branch_to_rebase.inner.full());
        reset(branch_to_rebase.tmp_branch_name(), true);
    } else {
        println!("Cherry-picking changes onto branch {}", onto);
        cherry_pick(
            branch_to_rebase.inner.start(),
            branch_to_rebase.inner.full(),
            strategy,
        );
    }
    finish_rebase(branch_to_rebase);
}

pub(crate) fn fixup_rebase() {
    let br = TmpBranchWrapper::new_from_tmp_branch(current_branch());

    finish_rebase(br);
}

fn finish_rebase(br: TmpBranchWrapper) {
    checkout(&br.inner.start());
    reset(br.tmp_start_branch_name(), true);

    checkout(&br.inner.full());
    reset(br.tmp_branch_name(), true);

    delete_branch_local(&br.tmp_branch_name());
    delete_branch_local(&br.tmp_start_branch_name())
}

const TMP_PREFIX: &str  = "_tmp_-";

struct TmpBranchWrapper {
    inner: ParsedBranch
}

impl TmpBranchWrapper {
    fn new_from_tmp_branch(branch: String) -> TmpBranchWrapper {
        let orig_br = branch_from_tmp(branch);
        TmpBranchWrapper::new(parse_branch(orig_br))
    }

    fn new(p: ParsedBranch) -> TmpBranchWrapper {
        TmpBranchWrapper {
            inner: p,
        }
    }

    fn tmp_branch_name(&self) -> String {
        tmp_for_branch_name(self.inner.full())
    }

    fn tmp_start_branch_name(&self) -> String {
        tmp_for_branch_name(self.inner.start())
    }
}


fn tmp_for_branch_name(branch: String) -> String {
    format!("{}{}", TMP_PREFIX, branch).to_string()
}

fn branch_from_tmp(branch: String) -> String {
    if let Some(suffix) = branch.strip_prefix(TMP_PREFIX) {
        return suffix.to_string()
    }
    panic!("Expected to be in tmp branch/rebase")
}

fn branches_are_equivalent(br1: String, br2: String) -> bool {
    get_commit_hash(br1) == get_commit_hash(br2)
}