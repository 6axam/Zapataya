#!/usr/bin/python3
import ctypes, json, os, pathlib, subprocess, sys
p = pathlib.Path.home()/'.local/share/comma-sound'
state = p/'state'
try:
    enabled, volume = map(int, state.read_text().split())
except (OSError, ValueError):
    enabled, volume = 1, 50
cmd = sys.argv[1]
if cmd == 'toggle': enabled = 1-enabled
elif cmd == 'up': volume = min(100, volume+10)
elif cmd == 'down': volume = max(0, volume-10)
elif cmd == 'play':
    if not enabled or not volume: sys.exit(0)
    try:
        devices = json.loads(subprocess.check_output(['hyprctl','devices','-j'], timeout=1))['keyboards']
        k = next(k for k in devices if k['main'])
        class Names(ctypes.Structure):
            _fields_ = [(x, ctypes.c_char_p) for x in ('rules','model','layout','variant','options')]
        x = ctypes.CDLL('libxkbcommon.so.0')
        x.xkb_context_new.argtypes = [ctypes.c_int]; x.xkb_context_new.restype = ctypes.c_void_p
        x.xkb_keymap_new_from_names.argtypes = [ctypes.c_void_p, ctypes.POINTER(Names), ctypes.c_int]; x.xkb_keymap_new_from_names.restype = ctypes.c_void_p
        x.xkb_keymap_key_get_syms_by_level.argtypes = [ctypes.c_void_p, ctypes.c_uint32, ctypes.c_uint32, ctypes.c_uint32, ctypes.POINTER(ctypes.POINTER(ctypes.c_uint32))]; x.xkb_keymap_key_get_syms_by_level.restype = ctypes.c_int
        names = Names(*[(k.get(z) or '').encode() for z in ('rules','model','layout','variant','options')])
        context = x.xkb_context_new(0); keymap = x.xkb_keymap_new_from_names(context, ctypes.byref(names), 0)
        syms = ctypes.POINTER(ctypes.c_uint32)()
        count = x.xkb_keymap_key_get_syms_by_level(keymap, int(sys.argv[2]), k['active_layout_index'], int(sys.argv[3]), ctypes.byref(syms))
        if count < 1 or syms[0] != 0x2c: sys.exit(0)
        subprocess.Popen(['paplay', '--volume='+str(round(65536*volume/100)), str(p/'sound.wav')], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
    except (OSError, KeyError, StopIteration, ValueError, subprocess.SubprocessError): pass
    sys.exit(0)
else: sys.exit(1)
state.write_text(f'{enabled} {volume}\n')
