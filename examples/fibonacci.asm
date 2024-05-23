.org 0
start:
; x will be set in the emulator
jsr fib
jmp end
fib:
pha
txa
pha
cpx #$00
beq fib_0
cpx #$01
beq fib_1
dex
jsr fib
tya
dex
jsr fib
sty $f7
adc $f7
tay
pla
tax
pla
rts
fib_0:
ldy #$00
pla
tax
pla
rts
fib_1:
ldy #$01
pla
tax
pla
rts
end:
brk ; end the program by signalling the emulator it is done

.org 0xfffc
.word start ; Reset vector
.word 0x0000
