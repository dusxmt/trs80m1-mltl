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
;    $ zasm message.asm
;    $ trs80m1-mltl -i message.bin -b 0x6000 -s 0x6000
;
; Which will yield a message.cas file, which you can then use with your
; emulator of choice.
; 

#target		bin
#code	 	CODE,	0x6000


; Location of the video memory:
video_mem	equ	0x3C00

; Copy the new screen content into video memory:
		ld	hl, video_mem
		ld	de, message
		ld	b, 16		; Outer loop iteration count.
outer_loop:	ld	c, 64		; Inner loop iteration count.
inner_loop:	ld	a, (de)
		ld	(hl), a
		inc	de
		inc	hl
		dec	c
		jp	nz, inner_loop	; Loop until we underflow.
		dec	b
		jp	nz, outer_loop	; Same for the outer loop.

stuck:		jp	stuck


; The message to display on the screen:
message:	incbin "screen_content.dat"
