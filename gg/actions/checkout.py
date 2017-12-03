from typing import Optional

from gg.gateways.git.branch_checkout import git_checkout
from gg.gateways.git.branch_info import get_current_branch
from gg.lib.branch_name import (
    get_branch_for_feature_part,
    get_first_branch_for_feature,
    get_next_branch,
    get_previous_branch,
    parse_branch_name,
)
from gg.lib.log import logger

def checkout(direction = None, part = None, feature = None) -> int:
    if direction == True:
        current_branch = get_current_branch()
        if not current_branch:
            logger.error('no branch currently checked out, cannot checkout relative commit')
            return 1
        branch_to_checkout = get_next_branch(current_branch)
    elif direction == False:
        current_branch = get_current_branch()
        if not current_branch:
            logger.error('no branch currently checked out, cannot checkout relative commit')
            return 1
        branch_to_checkout = get_previous_branch(current_branch)
    elif part is not None and feature is not None:
        branch_to_checkout = get_branch_for_feature_part(feature, part)
    elif part is not None:
        current_branch = get_current_branch()
        if not current_branch:
            logger.error('no branch currently checked out, cannot checkout relative commit')
            return 1
        branch = parse_branch_name(current_branch)
        branch_to_checkout = get_branch_for_feature_part(branch.feature, part)
    elif feature is not None:
        branch_to_checkout = get_first_branch_for_feature(feature)
    else:
        logger.error('cannot determine branch to checkout')
        return 1

    if branch_to_checkout is None:
        logger.error('cannot determine branch to checkout')
        return 1

    git_checkout(branch_to_checkout)
    return 0
