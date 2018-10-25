from gg.gateways.git.branch_checkout import git_checkout
from gg.gateways.git.branch_delete import safe_git_delete_branch
from gg.gateways.git.branch_info import get_current_branch
from gg.gateways.git.branch_create import git_checkout_new_branch
from gg.gateways.git.cherry_pick import git_cherry_pick, git_cherry_pick_abort, git_cherry_pick_continue
from gg.gateways.git.commit_info import get_commit
from gg.gateways.git.reset import git_reset
from gg.lib.branch_name import get_prefix_branch_name, get_previous_branch
from gg.lib.log import logger

TMP_PREFIX = "_tmp_-"


def abort_rebase():
    tmp_branch = get_current_branch()
    branch = get_branch_from_tmp(tmp_branch)
    if branch is None:
        logger.error("Not on a tmp branch, can't abort rebase")
        return 1
    tmp_prefix_branch = get_tmp_branch_name(get_prefix_branch_name(branch))

    # Stop the rebase
    try:
        git_cherry_pick_abort()
    except Exception as e:
        raise e

    # Checkout old branch
    git_checkout(branch)

    # Delete the tmp branches
    safe_git_delete_branch(tmp_branch)
    safe_git_delete_branch(tmp_prefix_branch)

    return 0

def continue_rebase():
    tmp_branch = get_current_branch()
    branch = get_branch_from_tmp(tmp_branch)
    if branch is None:
        logger.error("Not on a tmp branch, can't continue rebase")
        return 1
    tmp_prefix_branch = get_tmp_branch_name(get_prefix_branch_name(branch))

    # continue the rebase
    try:
        git_cherry_pick_continue()
    except Exception as e:
        raise e

    finish_rebase(
        tmp_branch,
        tmp_prefix_branch=tmp_prefix_branch,
        to_rebase_branch=branch,
    )


def start_rebase(onto: str, strategy: str):
    current_branch = get_current_branch()
    if current_branch is None:
        logger.error("No branch checked out")
        return 1

    if onto is not None:
        return rebase_onto(to_rebase_branch=current_branch, rebase_onto_branch=str(onto), strategy=strategy)

    previous_branch = get_previous_branch(current_branch)
    if previous_branch is None:
        logger.error("Could not determine branch to rebase onto")
        return 1

    return rebase_onto(to_rebase_branch=current_branch, rebase_onto_branch=previous_branch, strategy=strategy)

def rebase_onto(to_rebase_branch: str, rebase_onto_branch: str, strategy: str) -> int:
    """Rebase (we'll actually use a cherry-pick) a branch onto another one"""
    prefix_branch = get_prefix_branch_name(to_rebase_branch)

    tmp_branch = get_tmp_branch_name(to_rebase_branch)
    tmp_prefix_branch = get_tmp_branch_name(prefix_branch)

    # Create a new tmp branch in "onto" and onto's prefix
    git_checkout(rebase_onto_branch)
    git_checkout_new_branch(tmp_prefix_branch)
    git_checkout(rebase_onto_branch)
    git_checkout_new_branch(tmp_branch)

    # Cherry-pick the changes
    if branches_are_equal(prefix_branch, to_rebase_branch):
        # no commits to pick, fastfwd the checkouts instead
        logger.info("No commits to rebase, Fast forwarding branches")
        git_checkout(prefix_branch)
        git_reset(tmp_branch, hard=True)
        git_checkout(to_rebase_branch)
        git_reset(tmp_branch, hard=True)
        git_checkout(tmp_branch)
    else:
        logger.info("Cherry-picking changes onto new branch")
        git_cherry_pick(start_ref=prefix_branch, end_ref=to_rebase_branch, strategy=(str(strategy)))

    finish_rebase(
        tmp_branch,
        tmp_prefix_branch,
        to_rebase_branch,
    )

def branches_are_equal(branch1, branch2):
    """Returns true if two branches are off of the same git hash"""
    return get_commit(branch1).hash == get_commit(branch2).hash

def finish_rebase(
        tmp_branch: str,
        tmp_prefix_branch: str,
        to_rebase_branch: str,
):
    """Finish/cleanup the rebase after the cherry-pick"""
    prefix_branch = get_prefix_branch_name(to_rebase_branch)

    # Reset the prefix and to_rebase branches
    git_checkout(prefix_branch)
    git_reset(new_ref=tmp_prefix_branch, hard=True)

    git_checkout(to_rebase_branch)
    git_reset(new_ref=tmp_branch, hard=True)

    # Cleanup the tmp branch
    safe_git_delete_branch(tmp_branch)
    safe_git_delete_branch(tmp_prefix_branch)

def get_tmp_branch_name(branch_name: str) -> str:
    return TMP_PREFIX + branch_name

def get_branch_from_tmp(tmp_branch_name: str) -> str:
    if tmp_branch_name.startswith(TMP_PREFIX):
        return tmp_branch_name[len(TMP_PREFIX):]

    logger.error("Not in a rebase, no temporary branch checked out")
    return None
