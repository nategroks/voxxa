import pyautogui
import logging

class KeyboardSlideController:
    def next_slide(self):
        print("[CTRL] Sending Right arrow key")
        try:
            pyautogui.press("right")
        except Exception as e:
            print(f"[CTRL] Error sending keypress: {e}")
