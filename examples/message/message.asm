; To the extent possible under law, Marek Benc has waived all copyright and
; related or neighboring rights to all parts of this example program.
;
; Full text: http://creativecommons.org/publicdomain/zero/1.0/legalcode
;
;
; This program is a slightly modified version of the dummy rom file
; from trs80m1-rs, but instead of asking the user for a real rom file, it
; displays a friendly, encouraging message.
;
; To assemble and link this program:
;
;    $ zasm -w message.asm
;    $ trs80m1-mltl -i message.bin -b <base address> -s <entry point>
;
; Which will yield a message.cas file, which you can then use with your
; emulator of choice.
;
; You can find the base and entry point addresses in the `global symbols'
; part of the message.lst file, once the assembly is complete it.
;
; `BASE' is the base address, and `start' is the entry point.
;
; The command-line options expect hexadecimal input, with or without an
; optional 0x prefix.
; 

#target		bin
#code	 	BASE,	0x4A00


; Location of the video memory:
video_mem	equ	0x3C00


; The message to display on the screen:
message:	incbin "screen_content.dat"


; Copy the message onto the screen:
start:		ld	hl, message
		ld	de, video_mem
		ld	bc, 16 * 64
		ldir
		halt
