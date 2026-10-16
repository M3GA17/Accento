#!/usr/bin/env python3
"""Run only under an isolated Xvfb server: exercises the real XI2 backend and target entry."""
import ctypes as c
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import tkinter as tk

class DeviceInfo(c.Structure):
    _fields_ = [('deviceid',c.c_int),('name',c.c_char_p),('use',c.c_int),('attachment',c.c_int),('enabled',c.c_int),('num_classes',c.c_int),('classes',c.c_void_p)]

x=c.CDLL('libX11.so.6');xi=c.CDLL('libXi.so.6');xt=c.CDLL('libXtst.so.6')
x.XOpenDisplay.argtypes=[c.c_char_p];x.XOpenDisplay.restype=c.c_void_p
x.XKeysymToKeycode.argtypes=[c.c_void_p,c.c_ulong];x.XKeysymToKeycode.restype=c.c_ubyte
x.XSync.argtypes=[c.c_void_p,c.c_int]
xi.XIQueryDevice.argtypes=[c.c_void_p,c.c_int,c.POINTER(c.c_int)];xi.XIQueryDevice.restype=c.POINTER(DeviceInfo)
xi.XIFreeDeviceInfo.argtypes=[c.POINTER(DeviceInfo)]
xi.XOpenDevice.argtypes=[c.c_void_p,c.c_ulong];xi.XOpenDevice.restype=c.c_void_p
xt.XTestFakeDeviceKeyEvent.argtypes=[c.c_void_p,c.c_void_p,c.c_uint,c.c_int,c.c_void_p,c.c_int,c.c_ulong]
display=x.XOpenDisplay(None)
assert display,'Xvfb DISPLAY required'
count=c.c_int();devices=xi.XIQueryDevice(display,0,c.byref(count))
keyboard=next((devices[i].deviceid for i in range(count.value) if devices[i].use==4 and b'XTEST' not in devices[i].name),None)
xi.XIFreeDeviceInfo(devices)
assert keyboard is not None,'Isolated Xvfb physical keyboard required'
device=xi.XOpenDevice(display,keyboard)
assert device

root=tk.Tk();root.title('Accento native input fixture');root.geometry('700x250+80+80')
tk.Label(root,text='Accento · isolated X11 typing test',font=('Sans',18)).pack(pady=25)
entry=tk.Entry(root,font=('Sans',26),width=28);entry.pack();root.update();entry.focus_force();root.update()
def wait(seconds):
    end=time.monotonic()+seconds
    while time.monotonic()<end:root.update();time.sleep(.005)
def key(symbol,down):
    code=x.XKeysymToKeycode(display,symbol)
    assert code
    xt.XTestFakeDeviceKeyEvent(display,device,code,int(down),None,0,0);x.XSync(display,0);wait(.04)
def clear():entry.delete(0,tk.END);entry.focus_force();root.update()
def gesture(navigate=None,cancel=False,quick=False,shift=False,hold=.45,capture=False):
    if shift:key(0xffe1,True)
    key(ord('e'),True);key(0x20,True);key(0x20,False)
    wait(.02 if quick else hold)
    if capture:
        from PIL import ImageGrab
        image=ImageGrab.grab();w,h=image.size
        destination=Path('docs/evidenze/cross-platform');destination.mkdir(parents=True,exist_ok=True)
        image.crop((w//2-90,h-168,w//2+90,h-64)).save(destination/'linux-picker.png')
    if navigate:key(navigate,True);key(navigate,False)
    if cancel:key(0xff1b,True);key(0xff1b,False)
    key(ord('e'),False)
    if shift:key(0xffe1,False)
    wait(.15)

executable=Path(sys.argv[1]).resolve()
with tempfile.TemporaryDirectory(prefix='accento-x11-') as profile:
    Path(profile,'settings.json').write_text(json.dumps({'schema_version':1,'delay_ms':300,'languages':['it'],'theme':'dark','ui_language':'en','excluded_apps':[]}))
    env={**os.environ,'ACCENTO_CONFIG_DIR':profile,'XDG_SESSION_TYPE':'x11'};env.pop('WAYLAND_DISPLAY',None)
    log=open(Path(profile,'app.log'),'w+')
    child=subprocess.Popen([str(executable),'--input-test'],env=env,stdout=log,stderr=log)
    try:
        wait(4);assert child.poll() is None,'App exited during startup'
        from PIL import ImageGrab
        window=subprocess.check_output(['xdotool','search','--onlyvisible','--name','Accento.*Settings'],text=True).splitlines()[0]
        geometry=dict(line.split('=',1) for line in subprocess.check_output(['xdotool','getwindowgeometry','--shell',window],text=True).splitlines())
        x0,y0,w,h=(int(geometry[k]) for k in ['X','Y','WIDTH','HEIGHT'])
        destination=Path('docs/evidenze/cross-platform');destination.mkdir(parents=True,exist_ok=True)
        ImageGrab.grab().crop((x0,y0,x0+w,y0+h)).save(destination/'linux-settings.png')
        clear();gesture(capture=True);assert entry.get()=='è',repr(entry.get())
        clear();gesture(navigate=0xff53);assert entry.get()=='é',repr(entry.get())
        clear();gesture(cancel=True);assert entry.get()=='e',repr(entry.get())
        clear();gesture(quick=True);assert entry.get()=='e ',repr(entry.get())
        clear();gesture(shift=True);assert entry.get()=='È',repr(entry.get())
        clear();gesture(hold=1.1);assert entry.get()=='è',repr(entry.get())
        clear();key(ord('e'),True);wait(.95);key(0x20,True);key(0x20,False);key(ord('e'),False);wait(.15)
        assert entry.get().endswith(' ') and set(entry.get().strip())=={'e'},repr(entry.get())
        clear();key(ord('e'),True);key(0x20,True);key(0x20,False);wait(.4)
        other=tk.Toplevel(root);other.geometry('400x160+820+80');second=tk.Entry(other);second.pack(pady=30);root.update();second.focus_force();wait(.15)
        key(ord('e'),False);wait(.15)
        assert second.get()=='' and entry.get()=='e',(entry.get(),second.get())
        other.destroy()
        print('PASS: X11 real application insertion, navigation, cancellation, quick-space restoration, uppercase, auto-repeat before/during gestures and focus-change cancellation.')
    except Exception:
        log.flush();log.seek(0);print(log.read(),file=sys.stderr);raise
    finally:
        child.terminate()
        try:child.wait(timeout=5)
        except subprocess.TimeoutExpired:child.kill();child.wait()
        log.close();root.destroy()
