# Runbook: Voxxa can't reach the presentation app

**For** the volunteer at the slides computer (Windows). It takes about five minutes.

## Symptoms

The light is **red** and says one of these (`<app>` is ProPresenter, FreeShow, OpenLP, OpenSong
or "Your presentation app"):

- "`<app>` not found. Voxxa can't change slides." Voxxa is not connected to the app you chose,
  for example because the app was closed, or because it needs a password, which Voxxa never
  saves.
- "`<app>` didn't take the last slide change: `<reason>`." The app refused or did not answer.
  The reason "focus your presentation app: Voxxa's own window has the keyboard" means Voxxa is
  pressing the arrow keys for you and its own window was in front.

With WorshipTools Presenter the light can also be amber and ask you to do something in
Presenter: see [WorshipTools Presenter](#worshiptools-presenter) below.

## Steps

1. Make sure the presentation app is open and a presentation is loaded.
2. **If Voxxa presses keys for you** (no app was found in the set-up guide): click into the
   presentation app's window right after you choose **Songs**, and leave it in front. Keys sent
   while Voxxa's window is in front are refused on purpose.
3. **If Voxxa talks to the app over the network:** in the app, check that its remote control or
   network option is on. Voxxa looks on this computer at the usual ports: ProPresenter 1025,
   FreeShow 5506, OpenLP 4316, OpenSong 8082. If the app runs on another computer, both computers
   must be on the same network, and **Host** is that computer's address, not 127.0.0.1.
4. In Voxxa, press **Advanced**, then the **Settings** tab. Under **Presentation App**, press
   **Auto-detect**, or pick your app, check the host and port, type the password if it has one,
   and press **Connect**.
5. Or press **Advanced > Set-up guide**, go to step 2 ("Find your presentation app"), and pick
   your app from the list. If it finds nothing, open the app and press **Search again**.
6. Press **Simple mode** and check the light. It turns amber or green once Voxxa is connected,
   and after the next slide change that reaches the app.

ProPresenter's WebSocket connection and OpenLP with a login need the password again after every
start of Voxxa, and so does OpenSong with an API key: enter the key again under **Presentation
App > API key (optional)** and press **Connect**. Voxxa never saves passwords or keys. Expect this
red light after each start until you connect in step 4.

While you fix it, move the slides by hand in your presentation app.

## WorshipTools Presenter

WorshipTools Presenter has no remote control on this computer, so Voxxa presses its arrow keys:
Right for the next slide, Left for the one before. The set-up guide finds Presenter when it is
running and says so in step 2. Choose it there, or in **Advanced > Settings** choose
**Keystroke** with the key profile **WorshipTools Presenter**.

- **Keep Presenter in front** during the service. Voxxa sends a key only while a Presenter window
  is the one in front. Otherwise it sends nothing, and the light is **amber**: "Bring WorshipTools
  Presenter to the front." Click into Presenter.
- **At a song change Voxxa presses nothing.** In Presenter, Right on a song's last slide goes to
  the next service item, which may be a reading or an announcement. So when the next song starts,
  the light is **amber**: "Put "`<song>`" live in Presenter." Click that song in Presenter. Voxxa
  then counts from its first slide. If you don't, the next Right goes to whatever item comes next
  in Presenter.
- **Voxxa can't blank Presenter.** At a prayer the light says "Voxxa can't clear Presenter's
  screen." Clear it by hand if you need to. F10 clears the lyrics in Presenter, but how to bring
  them back has not been checked yet, so Voxxa doesn't use it. F11 and F12 clear the background
  and everything, and Up, Down and the number pad's + and − change the service item: Voxxa never
  sends any of them. **Blank** in Voxxa says it is not supported.
- **Don't run Presenter as administrator.** Windows then drops Voxxa's keys without any message,
  and the slides never move. Start Presenter the normal way.

## When to Call for help

Call for help when the light is still red after step 6.

1. Press **Call for help** in Voxxa. It shows you every file it will save. Passwords are never
   in it. Nothing is sent.
2. Press **Save to Desktop**. Your e-mail program opens with a message that says which file to
   attach, and the folder with the file opens too.
3. Type your tech lead's address, drag the file into the message, and send it.

See also: [the quick-start](../quick-start.md).
