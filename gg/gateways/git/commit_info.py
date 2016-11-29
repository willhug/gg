from typing import List

from gg.gateways.git import git_cmd

class Commit:
    hash = None # type: str
    title = None # type: str
    author = None # type: str
    body = None # type: str
    def __init__(
        self, hash: str,
        title: str,
        author: str,
        body: str = None,
    ) -> None:
        self.hash = hash
        self.title = title
        self.author = author
        self.body = body

    def get_short_hash(self):
        return self.hash[:7]

def get_commit(ref: str ='HEAD') -> Commit:
    return get_commits(start_ref=ref + "^", end_ref=ref)[0]

def get_commits(start_ref: str, end_ref: str) -> List[Commit]:
    """Get a list of commit objects between the start and end ref start < ref <= end"""
    line_split = "^*^*^*^*^*^*^*^*^*"
    split_chars = "~*~*~*~"
    raw_commit_info = git_cmd['log', '--pretty=format:%H{s}%s{s}%an{s}%b{l}'.format(s=split_chars, l=line_split), "%s..%s" % (start_ref, end_ref)]()

    commits = []
    raw_commit_info_lines = raw_commit_info.split(line_split)
    for raw_commit_info_line in raw_commit_info_lines:
        raw_commit_info_line = raw_commit_info_line.strip()
        if len(raw_commit_info_line) == 0:
            continue
        commit_info = raw_commit_info_line.split(split_chars)
        commits.append(Commit(
            hash=commit_info[0],
            title=commit_info[1],
            author=commit_info[2],
            body=commit_info[3],
        ))
    return commits
