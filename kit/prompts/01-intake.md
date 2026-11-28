# 01 — Intake: the church profile, installs and the church folder

| | |
|---|---|
| Run with | skill `voxxa-church-setup`, or paste the prompt below into a Claude Code session opened in a checkout of the voxxa repository (tagged kit) on the booth PC |
| Who | the church's tech lead, with the agent |
| Time [est.] | 45–90 minutes |
| Cost band | S (about $3–8 of agent time) |
| Inputs | the tech lead's answers; `kit/templates/church-profile.template.json`; `kit/private-folder.md` |
| Outputs | `<church_root>\church-profile.json`; the empty church folder; the setup checklist ticked to § 3 |
| Acceptance | every field of the profile answered or marked "unknown"; uv, ffmpeg and voxxa installed by the tech lead; the folder matches `private-folder.md`; nothing outside `<church_root>` written except the kit settings file |
| Stops at | any install (the tech lead runs it), anything that would need the agent to read a song or press a key |

## Prompt

> You are helping a church set up voxxa, a booth-PC app that listens to worship singing and moves the slides in
> the church's presentation app. Read `kit/README.md`, `kit/private-folder.md`,
> `kit/templates/church-profile.template.json` and `kit/templates/church-setup-checklist.md` first.
>
> Rules for this whole session: never read, print or copy song lyrics or words voxxa heard; never send church data
> to any external service or connector; never press keys in the presentation app; never start voxxa listening;
> report numbers, counts, file names and line numbers only. The tech lead runs every install and every step that
> touches the presentation app, the microphone or a recording; you give the exact command and wait.
>
> 1. Interview the tech lead, one question at a time, to fill `church-profile.json`: presentation app and version;
>    whether it runs as administrator; the audio feed; whether the booth desktop is duplicated to the projector or
>    captured for a stream; whether services are streamed or recorded, monetised, or ask for gifts on the stream;
>    whether the church still holds a CCLI licence (`ccli.active`, no number needed); which licence kinds the
>    church will use (`kit/templates/licence-kinds.csv` explains each in plain words); whether recording consent
>    exists. Mark anything unknown as "unknown".
> 2. Give the install commands one by one (`winget install --id astral-sh.uv -e`; in a new window
>    `uv python install 3.12`; `winget install Gyan.FFmpeg`; the voxxa installer; once plan step V5c ships, the
>    voxxa-tools zip of the same version with its `.sha256` check, unzipped outside `<church_root>`, as
>    `kit/README.md` "Tools and when they arrive" says, and until then
>    tell the tech lead plainly that practice sessions, recording intake and the layout check must wait for it)
>    and the three hardware-check
>    commands from the checklist. Ask the tech lead to paste only the CPU name and cores, the GPU name and memory,
>    and `True`/`False` for the Vulkan loader. Fill `hardware` and propose a class (G, C6 or C4) with the reason.
> 3. Ask where `<church_root>` should be (default `%LOCALAPPDATA%\voxxa-church\`). Refuse a path inside OneDrive,
>    Dropbox, Google Drive, a network share, a code repository, roaming `%APPDATA%`, voxxa's install folder
>    `%LOCALAPPDATA%\voxxa\` or voxxa's app-data folder `%LOCALAPPDATA%\com.voxxa.app\`. Create the empty folders
>    from `private-folder.md`, the `.ignore` file (`*`) and `README.md` ("Private. Never leaves this PC."). Write
>    `church-profile.json` there.
> 4. Offer `kit/templates/agent-settings.example.md` and explain each line; the tech lead decides.
> 5. Tick the setup checklist to the end of § 3 and list what is left (songs, layout, bindings) with the next prompt
>    to run (02).
>
> Finish with: the folder path, the profile's unknown fields, and the hardware class. Nothing else.
