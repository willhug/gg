from typing import List, Set

from gg.gateways.git.branch_get import get_all_branches

REALLY_BIG_INT = 9223372036854775807
START_BRANCH_PREFIX = "_start_-"

class Branch:
    feature = None # type: str
    part = None # type: float
    change = None # type: str
    def __init__(self, feature: str, part: float = 0.0, change: str = None) -> None:
        self.feature = feature
        self.part = part
        self.change = change

def create_branch_name(feature: str, change: str, part: float) -> str:
    """Creates a branch name for the feature, change, and part number"""
    new_branch_name = feature

    if part is None:
        new_branch_name = new_branch_name + "-part_1.0"
    else:
        new_branch_name = new_branch_name + "-part_" + str(part)

    if change is not None:
        new_branch_name = new_branch_name + "-" + change

    return new_branch_name

def get_next_branch(branch_name: str) -> str:
    """Get the next branch/change in this feature"""
    branch = parse_branch_name(branch_name)

    next_branches = get_branches_in_range(
        branch.feature,
        branch.part + 0.01, # Unfortunate hack because I wanted to allow people to insert fractional branches between existing ones
        REALLY_BIG_INT,
    )

    if len(next_branches) == 0:
        return None

    return next_branches[0]

def get_previous_branch(branch_name: str) -> str:
    """Get the previous branch/change in this feature"""
    branch = parse_branch_name(branch_name)

    if branch.part == 0:
        return None

    previous_branches = get_branches_in_range(branch.feature, 0, branch.part)

    if len(previous_branches) == 0:
        return None

    return previous_branches[-1]

def get_first_branch_for_feature(feature_name: str) -> str:
    """Get the first branch of a feature stack"""
    branches = get_branches_in_range(feature_name, 0, REALLY_BIG_INT)
    if len(branches) == 0:
        return None
    return branches[0]

def get_branches_for_feature(feature: str) -> List[str]:
    """Get all the main branches for a feature"""
    return get_branches_in_range(feature, 0, REALLY_BIG_INT, reverseSort=True)

def get_branches_in_range(feature: str, start: float, end: float, reverseSort: bool = False) -> List[str]:
    """Get the branches in a particular range (start <= branch < end) that start with feature_name"""
    branch_names = []
    for branch_name in get_all_branches():
        if not branch_name.startswith(feature):
            continue

        branch = parse_branch_name(branch_name)
        if branch.feature != feature:
            continue

        approx_part = int(branch.part * 1000)
        approx_start = int(start * 1000)
        approx_end = int(end * 1000)
        if approx_part >= approx_start and approx_part < approx_end:
            branch_names.append(branch_name)

    def get_branch_part(branch_name):
        branch = parse_branch_name(branch_name)
        return branch.part
    branch_names.sort(key=get_branch_part, reverse=reverseSort)

    return branch_names

def get_branch_for_feature_part(feature_name: str, part: float) -> str:
    """Get a matching branch name for the provided feature and part (or return None)"""
    for branch_name in get_all_branches():
        if not branch_name.startswith(feature_name):
            continue
        branch = parse_branch_name(branch_name)
        if branch.feature == feature_name and int(branch.part * 1000) == int(part * 1000):
            return branch_name
    return None

def get_all_features() -> List[str]:
    """Get all the features in the branch"""
    features = []
    for branch_name in get_all_branches():
        if not branch_name.startswith(START_BRANCH_PREFIX):
            continue
        branch = parse_branch_name(branch_name[len(START_BRANCH_PREFIX):])
        if branch.feature not in features:
            features.append(branch.feature)
    return features

def parse_branch_name(branch_name: str) -> Branch:
    """Parses a branch name to get feature, part number, and change name"""
    branch_parts = branch_name.split("-part_", 1)
    feature = branch_parts[0]

    # e.g. "branch_name"
    if len(branch_parts) == 1:
        return Branch(feature)

    suffix_parts = branch_parts[1].split('-', 1)
    part = float(suffix_parts[0])

    # e.g. "branch_name-part#1.0"
    if len(suffix_parts) == 1:
        return Branch(feature, part)

    # e.g. "branch_name-part#1.0-some_change"
    change = suffix_parts[1]
    return Branch(feature, part, change)

def get_prefix_branch_name(branch_name: str) -> str:
    """
    A prefix branch is a marker for where the branch started
    when we determine what constitutes a "change" will will
    take the branch_name compared to it's "prefix"

    :param branch_name: full branch name
    :return: prefix branch name
    """
    return "%s%s" % (START_BRANCH_PREFIX, branch_name)
