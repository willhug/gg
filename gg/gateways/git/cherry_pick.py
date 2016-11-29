from plumbum import FG

from gg.gateways.git import git_cmd

def git_cherry_pick(start_ref: str, end_ref: str):
    """Cherry pick a series of changes onto the current checkout"""
    cmd = git_cmd['cherry-pick', '%s..%s' % (start_ref, end_ref)]

    (cmd) & FG

def git_cherry_pick_continue():
    """Cherry pick --continue"""
    cmd = git_cmd['cherry-pick', '--continue']

    (cmd) & FG

def git_cherry_pick_abort():
    """Cherry pick --continue"""
    cmd = git_cmd['cherry-pick', '--abort']

    (cmd) & FG
