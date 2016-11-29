from gg.gateways.git import git_cmd

def get_current_branch():
    command = git_cmd['rev-parse', '--abbrev-ref', 'HEAD']
    branch = command().strip(' \t\n\r')
    if branch == 'HEAD':
        return None
    return branch
