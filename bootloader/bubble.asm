; bubble.asm - bubble sort in a 512-byte boot sector.
; Draws 16 bars in text mode, beeps the PC speaker on every swap.
; Build: nasm -f bin bubble.asm -o bubble.bin

bits 16
org 0x7c00

N       equ 16              ; number of bars
H       equ 20              ; tallest bar
arr     equ 0x7e00          ; bar heights live just past the boot sector
sorted  equ 0x7e10          ; index where the sorted tail starts
DELAY   equ 70000           ; microseconds per step

start:
    xor ax, ax
    mov ds, ax
    mov ss, ax
    mov sp, 0x7c00
    cld
    mov ax, 0x0003          ; 80x25 colour text mode, clears screen
    int 0x10
    mov ah, 1               ; hide the cursor
    mov cx, 0x2000
    int 0x10
    push 0xb800
    pop es

    ; fill the array with pseudo-random heights, seeded from the BIOS clock
    xor ah, ah
    int 0x1a
    mov ax, dx
    mov di, arr
    mov cx, N
.fill:
    imul ax, ax, 25173      ; simple LCG
    add ax, 13849
    push ax
    mov al, ah
    xor ah, ah
    mov bl, H
    div bl
    inc ah                  ; height 1..H
    mov [di], ah
    inc di
    pop ax
    loop .fill

    mov bp, 0xff00          ; no highlighted pair
    mov word [sorted], N
    call draw
    call pause
    call pause

; ---- bubble sort ----
.outer:
    xor dl, dl              ; swapped this pass?
    xor bp, bp
.inner:
    mov ax, bp
    inc ax
    cmp ax, [sorted]
    jae .endpass
    call draw               ; show the pair being compared
    call pause
    mov al, [arr+bp]
    mov ah, [arr+bp+1]
    cmp al, ah
    jbe .next
    mov [arr+bp], ah        ; swap
    mov [arr+bp+1], al
    inc dl
    call draw
    call beep               ; pitch follows the bar that moved right
.next:
    inc bp
    jmp .inner
.endpass:
    dec word [sorted]       ; largest bar is now in place
    test dl, dl
    jnz .outer

; ---- done: everything green, play a rising sweep ----
    mov word [sorted], 0
    mov bp, 0xff00
    call draw
    mov si, arr
    mov cx, N
.sweep:
    lodsb
    call beep
    loop .sweep
.halt:
    hlt
    jmp .halt

; ---- draw all bars; bp and bp+1 are drawn red ----
draw:
    pusha
    xor di, di
    mov dh, 25              ; level of the current row (top row = 25)
.row:
    xor si, si
.col:
    mov ax, si
    mov bl, 5
    div bl                  ; al = bar index, ah = column inside the bar
    mov cx, 0x0020          ; blank cell
    cmp ah, 4
    je .put                 ; gap between bars
    xor bh, bh
    mov bl, al
    cmp dh, [arr+bx]
    ja .put                 ; above this bar
    mov ch, 0x0f            ; white = unsorted
    cmp bx, [sorted]
    jb .notsorted
    mov ch, 0x0a            ; green = sorted
.notsorted:
    cmp bx, bp
    je .red
    dec bx
    cmp bx, bp
    jne .block
.red:
    mov ch, 0x0c            ; red = being compared
.block:
    mov cl, 0xdb            ; full block character
.put:
    mov ax, cx
    stosw
    inc si
    cmp si, 80
    jb .col
    dec dh
    jnz .row
    mov si, title           ; title on row 1
    mov di, 160*1 + 2*34
.title:
    lodsb
    test al, al
    jz .done
    mov ah, 0x0e
    stosw
    jmp .title
.done:
    popa
    ret

; ---- beep with pitch from al (1..H), then silence ----
beep:
    pusha
    mov bl, al
    mov al, 0xb6            ; PIT channel 2, square wave
    out 0x43, al
    xor al, al
    out 0x42, al            ; divisor low byte
    mov al, 25
    sub al, bl              ; taller bar = smaller divisor = higher pitch
    out 0x42, al            ; divisor high byte
    in al, 0x61
    or al, 3                ; speaker on
    out 0x61, al
    call pause
    in al, 0x61
    and al, 0xfc            ; speaker off
    out 0x61, al
    popa
    ret

; ---- wait DELAY microseconds ----
pause:
    pusha
    mov ah, 0x86
    mov cx, DELAY >> 16
    mov dx, DELAY & 0xffff
    int 0x15
    popa
    ret

title db "BUBBLE SORT", 0

times 510 - ($ - $$) db 0
dw 0xaa55
