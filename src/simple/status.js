// The Simple-mode status light (ADR-017): one colour and one plain sentence
// from signals the GUI already receives. The first matching row wins.
//   micLost        listening-stopped arrived with disconnected: true
//   startError     start_listening refused (its message)
//   presenterError presenter-error arrived since the last slide was sent
//   connected      get_presenter_info().connected, and its kind is the one last
//                  chosen (first run or Advanced), so a failed reconnect at start-up shows red
//   listening      the GUI's listening flag
//   machine        the last machine-state event's state
export function light(s) {
  if (s.micLost) return ["red", "Can't hear the microphone. Check the cable, then switch to Songs again."];
  if (s.startError) return ["red", `Couldn't start: ${s.startError}`];
  if (s.presenterError || s.connected === false) {
    return ["red", "Your presentation app isn't answering. Check that it is open, or connect it under Advanced."];
  }
  if (!s.listening) return ["grey", "Off. Switch to Songs when the music starts."];
  if (s.machine === "singing" || s.machine === "inter_verse_silence") return ["green", "Following the song."];
  if (s.machine === "blank_hold") return ["amber", "Screen blanked: what I hear doesn't match your songs."];
  return ["amber", "Listening. Waiting for a song I know."];
}
