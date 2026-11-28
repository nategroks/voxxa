# Agent settings for church setup (example)

When an AI agent (Claude Code) helps set up a church, it works in a checkout of the voxxa repository on the
booth PC. These settings make it harder for the agent to reach the church's private data and services that hold
personal data. They do not make it impossible.

**Permission patterns are a seatbelt, not a lock; the real controls are (1) never start an agent with
`<church_root>` as its working or added directory, (2) skills that never ask for content, (3) the numbers-only
tools.** A shell command can still reach the folder in ways no pattern matches: through a variable, a relative
path after `cd`, a script, or one shell started from another (for example `powershell -Command` from Bash).

**[UNVERIFIED]** The exact permission-rule syntax below must be checked against the current Claude Code settings
documentation (for example with its `update-config` skill) before use. The intent is what matters.

Save as `.claude/settings.local.json` in the repository checkout on the booth PC (local only, never committed):

```json
{
  "permissions": {
    "deny": [
      "mcp__claude_ai_Planning_Center",
      "mcp__claude_ai_Google_Drive",
      "mcp__claude_ai_Gmail",
      "Read(//<church_root>/songs/**)",
      "Read(//<church_root>/services/*.setlist.json)",
      "Read(//<church_root>/sessions/**)",
      "Read(//<church_root>/practice/**)",
      "Read(//<church_root>/recordings/**)",
      "Read(//<church_root>/fixtures/**)",
      "Read(//<church_root>/inbox/**)",
      "Read(//<church_root>/consent/**)",
      "Read(//<church_root>/reports/**)",
      "Read(//<church_root>/licences/**)",
      "Grep(//<church_root>/**)",
      "Bash(cat *<church_root>*)",
      "Bash(type *<church_root>*)",
      "Bash(head *<church_root>*)",
      "Bash(tail *<church_root>*)",
      "Bash(less *<church_root>*)",
      "Bash(more *<church_root>*)",
      "Bash(grep *<church_root>*)",
      "Bash(rg *<church_root>*)",
      "Bash(cp *<church_root>*)",
      "Bash(copy *<church_root>*)",
      "Bash(xcopy *<church_root>*)",
      "Bash(robocopy *<church_root>*)",
      "PowerShell(Get-Content *<church_root>*)",
      "PowerShell(gc *<church_root>*)",
      "PowerShell(cat *<church_root>*)",
      "PowerShell(type *<church_root>*)",
      "PowerShell(Select-String *<church_root>*)",
      "PowerShell(sls *<church_root>*)",
      "PowerShell(Copy-Item *<church_root>*)",
      "PowerShell(cp *<church_root>*)",
      "PowerShell(Import-Csv *<church_root>*)",
      "Bash(git push:*)",
      "Bash(git remote:*)"
    ]
  }
}
```

How to fill in `<church_root>`:

- In the `Read` and `Grep` rules, write the real path in forward-slash form after `//`, for example
  `//C:/dev/voxxa-church-private/pilot/songs/**` on the founder's development booth. For a new church, expand
  `%LOCALAPPDATA%\voxxa-church` to the full path the same way.
- In the `Bash` and `PowerShell` rules, write the end of the folder's path, which matches however the rest of the
  path is typed (a full path, `%LOCALAPPDATA%`, `$env:LOCALAPPDATA`): `voxxa-church` for a new church (one name
  with no separator, so each rule once is enough), `voxxa-church-private\pilot` on the founder's development
  booth. Write a rule whose path has a separator twice, once with `\` (in JSON, `\\`) and once with `/`, because
  commands use both.

The signed consent forms (`consent\`), the reports (`reports\`) and the licence notes (`licences\`, if present)
are denied too: the three `Read` rules above name them, and the `Grep` rule and the `Bash` and `PowerShell`
patterns already cover the whole of `<church_root>`, these three folders included. An agent may write a new review
or licence note there when a prompt says so, but never reads one back.

The agent may still list folder contents, create empty folders, read `church-profile.json`, `layout.json`,
`services\*.service.json` and `songs.csv` (none holds lyrics), and run tools that print counts.

Every kit skill also says, in its own words: never send church data to any external service; numbers, counts,
file names and line numbers only; never read or print lyrics or heard words; never press keys in the
presentation app.
