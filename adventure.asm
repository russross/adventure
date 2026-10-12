; Adventure: address-labeled disassembly of ADVNTURE.BIN.
; Assemble: dasm adventure.asm -f3 -obuild/adventure.bin -lbuild/adventure.lst -sbuild/adventure.sym
; Logical origin $F000; the 6507 sees the corresponding $1000-$1FFF cartridge window.
; 1236 reachable instructions / 2492 code bytes; all remaining bytes are preserved explicitly.
; L_xxxx = basic-block entry; D_xxxx = noncode/table boundary (including interior references).
; Numeric RAM operands retain their original addresses; TIA/RIOT names describe hardware.
; Every instruction comment includes its original address and bytes.
; Control-flow assumptions: ordinary calls/returns, depth <= 5, no overlapping code/data.
; The only unreachable code-shaped fragment is $F003-$F007, emitted as bytes.
; Authoritative source for editing: this file. tools/analyze.py independently traces the ROM.
; Hardware timing reference: Atari Stella Programmer Guide (1979),
; https://www.atariage.com/2600/programming/2600_101/docs/stella.html
; Timing uses 76 CPU cycles per scanline; cycle N means instruction completion N cycles
; after WSYNC release. The WSYNC store is 3 cycles plus the wait to the next boundary.
; Read-indexed page crossings cost one additional cycle; branch-page crossings do too.

        processor 6502

; TIA write registers and the read mirrors used in this ROM.
VSYNC        = $00
VBLANK       = $01
WSYNC        = $02
RSYNC        = $03
NUSIZ0       = $04
NUSIZ1       = $05
COLUP0       = $06
COLUP1       = $07
COLUPF       = $08
COLUBK       = $09
CTRLPF       = $0A
PF0          = $0D
PF1          = $0E
PF2          = $0F
RESP0        = $10
AUDC0        = $15
AUDF0        = $17
AUDV0        = $19
AUDV1        = $1A
GRP0         = $1B
GRP1         = $1C
ENAM0        = $1D
ENAM1        = $1E
ENABL        = $1F
HMP0         = $20
VDELP1       = $26
HMOVE        = $2A
HMCLR        = $2B
CXCLR        = $2C
CXP0FB       = $32
CXP1FB       = $33
CXM0FB       = $34
CXM1FB       = $35
CXBLPF       = $36
CXPPMM       = $37
INPT4        = $3C
SWCHA        = $0280
SWCHB        = $0282
INTIM        = $0284
TIM64T       = $0296

        org $F000


; -----------------------------------------------------------------------------
; Vector entry: jump to hardware initialization at $F2EF.
L_F000:
        jmp    L_F2EF               ; $F000: 4C EF F2

; Unreachable startup-shaped bytes: SEI; CLD; JMP $F306. No control-flow edge enters here.
D_F003:
        .byte $78,$D8,$4C,$06,$F3                              ; $F003-$F007

; -----------------------------------------------------------------------------
; Display a frame using prepared room and object state; start the overscan timer before returning.
; JSR sites: $F331, $F348, $F35F, $F368, $F401.
; FRAME DISPLAY / TWO-LINE KERNEL SETUP.
; Pointers: $80/$81 = playfield; $82/$83 = P0 bitmap; $84/$85 = P1 bitmap.
; $86/$87 = P0 x/y; $88/$89 = P1 x/y; $8B/$8C = ball x/y.
; $8E is the row countdown; $8F is the playfield offset; $90/$91 are bitmap offsets.
; VDELP1 enables delayed P1 graphics: line A writes GRP1; line B's GRP0 write
; transfers that value to P1's delayed graphics latch. Each bitmap byte spans two lines.
L_F008:
        sta.b  HMCLR                ; $F008: 85 2B
        lda.b  $86                  ; $F00A: A5 86
        ldx    #$00                 ; $F00C: A2 00
        jsr    L_F0D2               ; $F00E: 20 D2 F0
; Call L_F0D2; continue at L_F018.
L_F011:
        lda.b  $88                  ; $F011: A5 88
        ldx    #$01                 ; $F013: A2 01
        jsr    L_F0D2               ; $F015: 20 D2 F0
; Call L_F0D2; continue at L_F01F.
L_F018:
        lda.b  $8B                  ; $F018: A5 8B
        ldx    #$04                 ; $F01A: A2 04
        jsr    L_F0D2               ; $F01C: 20 D2 F0
; Horizontal boundary after positioning; HMOVE follows immediately at the start of the next line.
L_F01F:
        sta.b  WSYNC                ; $F01F: 85 02    ; horizontal wait
        sta.b  HMOVE                ; $F021: 85 2A
        sta.b  CXCLR                ; $F023: 85 2C
        lda.b  $8C                  ; $F025: A5 8C
        sec                         ; $F027: 38
        sbc    #$04                 ; $F028: E9 04
        sta.b  $8D                  ; $F02A: 85 8D
; VERTICAL-BLANK TIMER WAIT: poll INTIM until zero (7 cycles per taken iteration).
L_F02C:
        lda.w  INTIM                ; $F02C: AD 84 02
        bne    L_F02C               ; $F02F: D0 FB    ; 2/3 cycles
; Initialize bitmap offsets and preload the first playfield triple before unblanking.
L_F031:
        lda    #$00                 ; $F031: A9 00
        sta.b  $90                  ; $F033: 85 90
        sta.b  $91                  ; $F035: 85 91
        sta.b  $8F                  ; $F037: 85 8F
        sta.b  GRP1                 ; $F039: 85 1C
        lda    #$01                 ; $F03B: A9 01
        sta.b  VDELP1               ; $F03D: 85 26
        lda    #$68                 ; $F03F: A9 68
        sta.b  $8E                  ; $F041: 85 8E
        ldy.b  $8F                  ; $F043: A4 8F
        lda    ($80),y              ; $F045: B1 80    ; 5 cycles (+1 on page crossing)
        sta.b  PF0                  ; $F047: 85 0D
        iny                         ; $F049: C8
        lda    ($80),y              ; $F04A: B1 80    ; 5 cycles (+1 on page crossing)
        sta.b  PF1                  ; $F04C: 85 0E
        iny                         ; $F04E: C8
        lda    ($80),y              ; $F04F: B1 80    ; 5 cycles (+1 on page crossing)
        sta.b  PF2                  ; $F051: 85 0F
        iny                         ; $F053: C8
        sty.b  $8F                  ; $F054: 84 8F
; KERNEL FIRST-LINE BOUNDARY: WSYNC releases into $F058 at CPU cycle 0.
; Clear VBLANK at cycle 5; enter $F072 at cycle 8, skipping the first P1 fetch.
L_F056:
        sta.b  WSYNC                ; $F056: 85 02    ; horizontal wait
L_F058:
        lda    #$00                 ; $F058: A9 00
        sta.b  VBLANK               ; $F05A: 85 01
        jmp    L_F072               ; $F05C: 4C 72 F0
; TWO-LINE KERNEL BACK EDGE: compute the P1 vertical comparison before the next WSYNC.
L_F05F:
        lda.b  $8E                  ; $F05F: A5 8E
        sec                         ; $F061: 38
        sbc.b  $89                  ; $F062: E5 89
; KERNEL LINE A BOUNDARY: WSYNC releases into $F066 at CPU cycle 0.
; The comparison flags survive WSYNC; update P1 only after its vertical threshold.
; Paths to line B cost 41-75 cycles on refresh rows or 42-76 on ordinary rows,
; including the WSYNC store. Sprite streams do not incur page-crossing penalties.
L_F064:
        sta.b  WSYNC                ; $F064: 85 02    ; horizontal wait
L_F066:
        bpl    L_F072               ; $F066: 10 0A    ; 2/3 cycles
; Conditional paths join or continue at L_F072 and L_F070.
L_F068:
        ldy.b  $91                  ; $F068: A4 91
        lda    ($84),y              ; $F06A: B1 84    ; 5 cycles (+1 on page crossing)
        sta.b  GRP1                 ; $F06C: 85 1C
        beq    L_F072               ; $F06E: F0 02    ; 2/3 cycles
; Fall through to L_F072.
L_F070:
        inc.b  $91                  ; $F070: E6 91
; Prepare P0 graphics in X; advance its bitmap offset only for a nonzero byte.
; On regular iterations the P1 path costs 3, 16, or 20 cycles before this block.
L_F072:
        ldx    #$00                 ; $F072: A2 00
        lda.b  $8E                  ; $F074: A5 8E
        sec                         ; $F076: 38
        sbc.b  $87                  ; $F077: E5 87
        bpl    L_F084               ; $F079: 10 09    ; 2/3 cycles
; Conditional paths join or continue at L_F084 and L_F082.
L_F07B:
        ldy.b  $90                  ; $F07B: A4 90
        lda    ($82),y              ; $F07D: B1 82    ; 5 cycles (+1 on page crossing)
        tax                         ; $F07F: AA
        beq    L_F084               ; $F080: F0 02    ; 2/3 cycles
; Fall through to L_F084.
L_F082:
        inc.b  $90                  ; $F082: E6 90
; Compute ball enable in Y: two means visible when ($8E-$8D)&$FC is zero.
L_F084:
        ldy    #$00                 ; $F084: A0 00
        lda.b  $8E                  ; $F086: A5 8E
        sec                         ; $F088: 38
        sbc.b  $8D                  ; $F089: E5 8D
        and    #$FC                 ; $F08B: 29 FC
        bne    L_F091               ; $F08D: D0 02    ; 2/3 cycles
; Fall through to L_F091.
L_F08F:
        ldy    #$02                 ; $F08F: A0 02
; Every sixteenth row uses the playfield-refresh path; other rows keep the previous triple.
L_F091:
        lda.b  $8E                  ; $F091: A5 8E
        and    #$0F                 ; $F093: 29 0F
        bne    L_F0BD               ; $F095: D0 26    ; 2/3 cycles
; KERNEL LINE B BOUNDARY (playfield refresh): resume $F099 at CPU cycle 0.
; Write ENABL at cycle 3 and GRP0 at cycle 6.
; Without indirect page crossings, PF0/PF1/PF2 writes complete at cycles 17/27/37.
L_F097:
        sta.b  WSYNC                ; $F097: 85 02    ; horizontal wait
L_F099:
        sty.b  ENABL                ; $F099: 84 1F
        stx.b  GRP0                 ; $F09B: 86 1B
        ldy.b  $8F                  ; $F09D: A4 8F
        lda    ($80),y              ; $F09F: B1 80    ; 5 cycles (+1 on page crossing)
        sta.b  PF0                  ; $F0A1: 85 0D
        iny                         ; $F0A3: C8
        lda    ($80),y              ; $F0A4: B1 80    ; 5 cycles (+1 on page crossing)
        sta.b  PF1                  ; $F0A6: 85 0E
        iny                         ; $F0A8: C8
        lda    ($80),y              ; $F0A9: B1 80    ; 5 cycles (+1 on page crossing)
        sta.b  PF2                  ; $F0AB: 85 0F
        iny                         ; $F0AD: C8
        sty.b  $8F                  ; $F0AE: 84 8F
; Finish this pair of scanlines and count down one logical row.
; The $68..$08 inclusive countdown gives 97 two-line iterations.
; Continue at $F05F; the next line-A WSYNC completes by cycle 66 on a refresh row
; or cycle 33 on an ordinary row (indirect page-cross penalties are additional).
L_F0B0:
        dec.b  $8E                  ; $F0B0: C6 8E
        lda.b  $8E                  ; $F0B2: A5 8E
        cmp    #$08                 ; $F0B4: C9 08
        bpl    L_F05F               ; $F0B6: 10 A7    ; 2/3 cycles
; KERNEL EXIT: $8E is now $07; its bit 1 blanks the display.
; This write occurs during the last line B; no extra WSYNC is issued here.
; VBLANK completes at cycle 24 on the ordinary exit or 57 on a refresh exit.
; The actual final row is $08, so it follows the ordinary exit.
L_F0B8:
        sta.b  VBLANK               ; $F0B8: 85 01
        jmp    L_F0C6               ; $F0BA: 4C C6 F0
; KERNEL LINE B BOUNDARY (ordinary row): resume $F0BF at CPU cycle 0.
; Write ENABL at cycle 3 and GRP0 at cycle 6, then join the row-countdown tail.
L_F0BD:
        sta.b  WSYNC                ; $F0BD: 85 02    ; horizontal wait
L_F0BF:
        sty.b  ENABL                ; $F0BF: 84 1F
        stx.b  GRP0                 ; $F0C1: 86 1B
        jmp    L_F0B0               ; $F0C3: 4C B0 F0
; Clear both player graphics and start a 32*64 CPU-cycle overscan interval (2048 cycles).
L_F0C6:
        lda    #$00                 ; $F0C6: A9 00
        sta.b  GRP1                 ; $F0C8: 85 1C
        sta.b  GRP0                 ; $F0CA: 85 1B
        lda    #$20                 ; $F0CC: A9 20
        sta.w  TIM64T               ; $F0CE: 8D 96 02
        rts                         ; $F0D1: 60

; -----------------------------------------------------------------------------
; Position one TIA object. A = horizontal coordinate; X = object number (0..4).
; JSR sites: $F00E, $F015, $F01C, $F375, $F37C.
L_F0D2:
        ldy    #$02                 ; $F0D2: A0 02
        sec                         ; $F0D4: 38
; Coarse positioning: subtract 15 until borrow; Y counts the corresponding delay intervals.
L_F0D5:
        iny                         ; $F0D5: C8
        sbc    #$0F                 ; $F0D6: E9 0F
        bcs    L_F0D5               ; $F0D8: B0 FB    ; 2/3 cycles
; Convert the residual coordinate to the signed fine-motion value in the high nibble.
L_F0DA:
        eor    #$FF                 ; $F0DA: 49 FF
        sbc    #$06                 ; $F0DC: E9 06
        asl                         ; $F0DE: 0A
        asl                         ; $F0DF: 0A
        asl                         ; $F0E0: 0A
        asl                         ; $F0E1: 0A
; POSITIONING HORIZONTAL WAIT: WSYNC releases into the calibrated delay loop.
        sty.b  WSYNC                ; $F0E2: 84 02    ; horizontal wait
; CALIBRATED HORIZONTAL DELAY: DEY/BPL uses 5 cycles per taken iteration, 4 on exit.
; With initial Y, the delay takes 5*Y+4 cycles before the indexed RESPx write.
L_F0E4:
        dey                         ; $F0E4: 88
        bpl    L_F0E4               ; $F0E5: 10 FD    ; 2/3 cycles
; Strobe the object reset, then store its fine horizontal motion; X chooses P0/P1/M0/M1/ball.
L_F0E7:
        sta.b  RESP0,x              ; $F0E7: 95 10
        sta.b  HMP0,x               ; $F0E9: 95 20
        rts                         ; $F0EB: 60

; -----------------------------------------------------------------------------
; Wait for overscan expiry, blank the display, generate three VSYNC lines, and start the vertical-blank timer.
; JSR sites: $F32B, $F33F, $F351, $F365, $F3FE.
; OVERSCAN TIMER WAIT: poll INTIM until zero (7 cycles per taken iteration).
L_F0EC:
        lda.w  INTIM                ; $F0EC: AD 84 02
        bne    L_F0EC               ; $F0EF: D0 FB    ; 2/3 cycles
; Return through the matching JSR continuation.
L_F0F1:
        lda    #$02                 ; $F0F1: A9 02
; SYNC HORIZONTAL BOUNDARY 0: resume $F0F5; assert VBLANK at cycle 3.
        sta.b  WSYNC                ; $F0F3: 85 02    ; horizontal wait
        sta.b  VBLANK               ; $F0F5: 85 01
; SYNC HORIZONTAL BOUNDARY 1: blank preparation line.
        sta.b  WSYNC                ; $F0F7: 85 02    ; horizontal wait
; SYNC HORIZONTAL BOUNDARY 2: blank preparation line.
        sta.b  WSYNC                ; $F0F9: 85 02    ; horizontal wait
; SYNC HORIZONTAL BOUNDARY 3: resume $F0FD; assert VSYNC at cycle 3.
        sta.b  WSYNC                ; $F0FB: 85 02    ; horizontal wait
        sta.b  VSYNC                ; $F0FD: 85 00
; SYNC HORIZONTAL BOUNDARY 4: VSYNC remains asserted.
        sta.b  WSYNC                ; $F0FF: 85 02    ; horizontal wait
; SYNC HORIZONTAL BOUNDARY 5: VSYNC remains asserted; load zero after the wait.
        sta.b  WSYNC                ; $F101: 85 02    ; horizontal wait
        lda    #$00                 ; $F103: A9 00
; SYNC HORIZONTAL BOUNDARY 6: resume $F107; clear VSYNC at cycle 3.
; The two VSYNC writes are exactly 3*76 = 228 CPU cycles apart.
        sta.b  WSYNC                ; $F105: 85 02    ; horizontal wait
        sta.b  VSYNC                ; $F107: 85 00
; Start a 42*64 CPU-cycle vertical-blank interval (2688 cycles); preparation runs before $F02C.
        lda    #$2A                 ; $F109: A9 2A
        sta.w  TIM64T               ; $F10B: 8D 96 02
        rts                         ; $F10E: 60

; -----------------------------------------------------------------------------
; Prepare the room playfield pointer and TIA colors, select two visible objects, and load their bitmap/position state.
; JSR sites: $F32E, $F36B.
L_F10F:
        lda.b  $8A                  ; $F10F: A5 8A
        jsr    L_F271               ; $F111: 20 71 F2
; Read the selected room record through $93/$94 and retain its playfield pointer.
L_F114:
        ldy    #$00                 ; $F114: A0 00
        lda    ($93),y              ; $F116: B1 93
        sta.b  $80                  ; $F118: 85 80
        ldy    #$01                 ; $F11A: A0 01
        lda    ($93),y              ; $F11C: B1 93
        sta.b  $81                  ; $F11E: 85 81
        lda.w  SWCHB                ; $F120: AD 82 02
        and    #$08                 ; $F123: 29 08
        beq    L_F133               ; $F125: F0 0C    ; 2/3 cycles
; Call L_F2D3; continue at L_F12E.
L_F127:
        ldy    #$02                 ; $F127: A0 02
        lda    ($93),y              ; $F129: B1 93
        jsr    L_F2D3               ; $F12B: 20 D3 F2
; Continue at L_F13C.
L_F12E:
        sta.b  COLUPF               ; $F12E: 85 08
        jmp    L_F13C               ; $F130: 4C 3C F1
; Call L_F2D3; continue at L_F13A.
L_F133:
        ldy    #$03                 ; $F133: A0 03
        lda    ($93),y              ; $F135: B1 93
        jsr    L_F2D3               ; $F137: 20 D3 F2
; Fall through to L_F13C.
L_F13A:
        sta.b  COLUPF               ; $F13A: 85 08
; Call L_F2D3; continue at L_F141.
L_F13C:
        lda    #$08                 ; $F13C: A9 08
        jsr    L_F2D3               ; $F13E: 20 D3 F2
; Call L_F235; continue at L_F158.
L_F141:
        sta.b  COLUBK               ; $F141: 85 09
        ldy    #$04                 ; $F143: A0 04
        lda    ($93),y              ; $F145: B1 93
        sta.b  CTRLPF               ; $F147: 85 0A
        and    #$C0                 ; $F149: 29 C0
        lsr                         ; $F14B: 4A
        lsr                         ; $F14C: 4A
        lsr                         ; $F14D: 4A
        lsr                         ; $F14E: 4A
        lsr                         ; $F14F: 4A
        sta.b  ENAM1                ; $F150: 85 1E
        lsr                         ; $F152: 4A
        sta.b  ENAM0                ; $F153: 85 1D
        jsr    L_F235               ; $F155: 20 35 F2
; Conditional paths join or continue at L_F168 and L_F15E.
L_F158:
        lda.b  $95                  ; $F158: A5 95
        cmp    #$00                 ; $F15A: C9 00
        beq    L_F168               ; $F15C: F0 0A    ; 2/3 cycles
; Conditional paths join or continue at L_F174 and L_F162.
L_F15E:
        cmp    #$5A                 ; $F15E: C9 5A
        bne    L_F174               ; $F160: D0 12    ; 2/3 cycles
; Conditional paths join or continue at L_F174 and L_F168.
L_F162:
        lda.b  $96                  ; $F162: A5 96
        cmp    #$00                 ; $F164: C9 00
        beq    L_F174               ; $F166: F0 0C    ; 2/3 cycles
; Fall through to L_F174.
L_F168:
        lda.b  $95                  ; $F168: A5 95
        sta.b  $D8                  ; $F16A: 85 D8
        lda.b  $96                  ; $F16C: A5 96
        sta.b  $95                  ; $F16E: 85 95
        lda.b  $D8                  ; $F170: A5 D8
        sta.b  $96                  ; $F172: 85 96
; Load the first displayed descriptor; follow its position, state, and graphics-selection pointers.
L_F174:
        ldx.b  $95                  ; $F174: A6 95
        lda.w  D_FF44,x             ; $F176: BD 44 FF
        sta.b  $93                  ; $F179: 85 93
        lda.w  D_FF45,x             ; $F17B: BD 45 FF
        sta.b  $94                  ; $F17E: 85 94
        ldy    #$01                 ; $F180: A0 01
        lda    ($93),y              ; $F182: B1 93
        sta.b  $86                  ; $F184: 85 86
        ldy    #$02                 ; $F186: A0 02
        lda    ($93),y              ; $F188: B1 93
        sta.b  $87                  ; $F18A: 85 87
        lda.w  D_FF46,x             ; $F18C: BD 46 FF
        sta.b  $93                  ; $F18F: 85 93
        lda.w  D_FF47,x             ; $F191: BD 47 FF
        sta.b  $94                  ; $F194: 85 94
        ldy    #$00                 ; $F196: A0 00
        lda    ($93),y              ; $F198: B1 93
        sta.b  $DC                  ; $F19A: 85 DC
        lda.w  D_FF48,x             ; $F19C: BD 48 FF
        sta.b  $93                  ; $F19F: 85 93
        lda.w  D_FF49,x             ; $F1A1: BD 49 FF
        sta.b  $94                  ; $F1A4: 85 94
        jsr    L_F2A1               ; $F1A6: 20 A1 F2
; Conditional paths join or continue at L_F1C5 and L_F1BA.
L_F1A9:
        iny                         ; $F1A9: C8
        lda    ($93),y              ; $F1AA: B1 93
        sta.b  $82                  ; $F1AC: 85 82
        iny                         ; $F1AE: C8
        lda    ($93),y              ; $F1AF: B1 93
        sta.b  $83                  ; $F1B1: 85 83
        lda.w  SWCHB                ; $F1B3: AD 82 02
        and    #$08                 ; $F1B6: 29 08
        beq    L_F1C5               ; $F1B8: F0 0B    ; 2/3 cycles
; Call L_F2D3; continue at L_F1C0.
L_F1BA:
        lda.w  D_FF4A,x             ; $F1BA: BD 4A FF
        jsr    L_F2D3               ; $F1BD: 20 D3 F2
; Continue at L_F1CD.
L_F1C0:
        sta.b  COLUP0               ; $F1C0: 85 06
        jmp    L_F1CD               ; $F1C2: 4C CD F1
; Call L_F2D3; continue at L_F1CB.
L_F1C5:
        lda.w  D_FF4B,x             ; $F1C5: BD 4B FF
        jsr    L_F2D3               ; $F1C8: 20 D3 F2
; Fall through to L_F1CD.
L_F1CB:
        sta.b  COLUP0               ; $F1CB: 85 06
; Call L_F2A1; continue at L_F209.
L_F1CD:
        lda.w  D_FF4C,x             ; $F1CD: BD 4C FF
        ora    #$10                 ; $F1D0: 09 10
        sta.b  NUSIZ0               ; $F1D2: 85 04
; Load the second displayed descriptor and prepare its TIA state in the same record format.
        ldx.b  $96                  ; $F1D4: A6 96
        lda.w  D_FF44,x             ; $F1D6: BD 44 FF
        sta.b  $93                  ; $F1D9: 85 93
        lda.w  D_FF45,x             ; $F1DB: BD 45 FF
        sta.b  $94                  ; $F1DE: 85 94
        ldy    #$01                 ; $F1E0: A0 01
        lda    ($93),y              ; $F1E2: B1 93
        sta.b  $88                  ; $F1E4: 85 88
        ldy    #$02                 ; $F1E6: A0 02
        lda    ($93),y              ; $F1E8: B1 93
        sta.b  $89                  ; $F1EA: 85 89
        lda.w  D_FF46,x             ; $F1EC: BD 46 FF
        sta.b  $93                  ; $F1EF: 85 93
        lda.w  D_FF47,x             ; $F1F1: BD 47 FF
        sta.b  $94                  ; $F1F4: 85 94
        ldy    #$00                 ; $F1F6: A0 00
        lda    ($93),y              ; $F1F8: B1 93
        sta.b  $DC                  ; $F1FA: 85 DC
        lda.w  D_FF48,x             ; $F1FC: BD 48 FF
        sta.b  $93                  ; $F1FF: 85 93
        lda.w  D_FF49,x             ; $F201: BD 49 FF
        sta.b  $94                  ; $F204: 85 94
        jsr    L_F2A1               ; $F206: 20 A1 F2
; Conditional paths join or continue at L_F225 and L_F21A.
L_F209:
        iny                         ; $F209: C8
        lda    ($93),y              ; $F20A: B1 93
        sta.b  $84                  ; $F20C: 85 84
        iny                         ; $F20E: C8
        lda    ($93),y              ; $F20F: B1 93
        sta.b  $85                  ; $F211: 85 85
        lda.w  SWCHB                ; $F213: AD 82 02
        and    #$08                 ; $F216: 29 08
        beq    L_F225               ; $F218: F0 0B    ; 2/3 cycles
; Call L_F2D3; continue at L_F220.
L_F21A:
        lda.w  D_FF4A,x             ; $F21A: BD 4A FF
        jsr    L_F2D3               ; $F21D: 20 D3 F2
; Continue at L_F22D.
L_F220:
        sta.b  COLUP1               ; $F220: 85 07
        jmp    L_F22D               ; $F222: 4C 2D F2
; Call L_F2D3; continue at L_F22B.
L_F225:
        lda.w  D_FF4B,x             ; $F225: BD 4B FF
        jsr    L_F2D3               ; $F228: 20 D3 F2
; Fall through to L_F22D.
L_F22B:
        sta.b  COLUP1               ; $F22B: 85 07
; Return through the matching JSR continuation.
L_F22D:
        lda.w  D_FF4C,x             ; $F22D: BD 4C FF
        ora    #$10                 ; $F230: 09 10
        sta.b  NUSIZ1               ; $F232: 85 05
        rts                         ; $F234: 60

; -----------------------------------------------------------------------------
; Rotate through object descriptors in nine-byte steps; select up to two objects in the current room.
; JSR sites: $F155.
L_F235:
        ldy.b  $9C                  ; $F235: A4 9C
        lda    #$A2                 ; $F237: A9 A2
        sta.b  $95                  ; $F239: 85 95
        sta.b  $96                  ; $F23B: 85 96
; Conditional paths join or continue at L_F247 and L_F245.
L_F23D:
        tya                         ; $F23D: 98
        clc                         ; $F23E: 18
        adc    #$09                 ; $F23F: 69 09
        cmp    #$A2                 ; $F241: C9 A2
        bcc    L_F247               ; $F243: 90 02    ; 2/3 cycles
; Fall through to L_F247.
L_F245:
        lda    #$00                 ; $F245: A9 00
; Conditional paths join or continue at L_F26A and L_F25A.
L_F247:
        tay                         ; $F247: A8
        lda.w  D_FF44,y             ; $F248: B9 44 FF
        sta.b  $93                  ; $F24B: 85 93
        lda.w  D_FF45,y             ; $F24D: B9 45 FF
        sta.b  $94                  ; $F250: 85 94
        ldx    #$00                 ; $F252: A2 00
        lda    ($93,x)              ; $F254: A1 93
        cmp.b  $8A                  ; $F256: C5 8A
        bne    L_F26A               ; $F258: D0 10    ; 2/3 cycles
; Conditional paths join or continue at L_F265 and L_F260.
L_F25A:
        lda.b  $95                  ; $F25A: A5 95
        cmp    #$A2                 ; $F25C: C9 A2
        bne    L_F265               ; $F25E: D0 05    ; 2/3 cycles
; Continue at L_F26A.
L_F260:
        sty.b  $95                  ; $F260: 84 95
        jmp    L_F26A               ; $F262: 4C 6A F2
; Continue at L_F26E.
L_F265:
        sty.b  $96                  ; $F265: 84 96
        jmp    L_F26E               ; $F267: 4C 6E F2
; Conditional paths join or continue at L_F23D and L_F26E.
L_F26A:
        cpy.b  $9C                  ; $F26A: C4 9C
        bne    L_F23D               ; $F26C: D0 CF    ; 2/3 cycles
; Return through the matching JSR continuation.
L_F26E:
        sty.b  $9C                  ; $F26E: 84 9C
        rts                         ; $F270: 60

; -----------------------------------------------------------------------------
; Compute $93/$94 = $FE1B + 9*A; $D8 holds the input room number.
; JSR sites: $F111, $F6A1, $F9E9.
L_F271:
        sta.b  $D8                  ; $F271: 85 D8
        sta.b  $93                  ; $F273: 85 93
        lda    #$00                 ; $F275: A9 00
        sta.b  $94                  ; $F277: 85 94
        clc                         ; $F279: 18
        rol.b  $93                  ; $F27A: 26 93
        rol.b  $94                  ; $F27C: 26 94
        rol.b  $93                  ; $F27E: 26 93
        rol.b  $94                  ; $F280: 26 94
        rol.b  $93                  ; $F282: 26 93
        rol.b  $94                  ; $F284: 26 94
        lda.b  $D8                  ; $F286: A5 D8
        clc                         ; $F288: 18
        adc.b  $93                  ; $F289: 65 93
        sta.b  $93                  ; $F28B: 85 93
        lda    #$00                 ; $F28D: A9 00
        adc.b  $94                  ; $F28F: 65 94
        sta.b  $94                  ; $F291: 85 94
        lda    #<D_FE1B             ; $F293: A9 1B
        clc                         ; $F295: 18
        adc.b  $93                  ; $F296: 65 93
        sta.b  $93                  ; $F298: 85 93
        lda    #>D_FE1B             ; $F29A: A9 FE
        adc.b  $94                  ; $F29C: 65 94
        sta.b  $94                  ; $F29E: 85 94
        rts                         ; $F2A0: 60

; -----------------------------------------------------------------------------
; Select the first graphics triple whose state ceiling is at least $DC; return its byte offset in Y.
; JSR sites: $F1A6, $F206.
L_F2A1:
        ldy    #$00                 ; $F2A1: A0 00
        lda.b  $DC                  ; $F2A3: A5 DC
; Conditional paths join or continue at L_F2B1 and L_F2A9.
L_F2A5:
        cmp    ($93),y              ; $F2A5: D1 93
        bcc    L_F2B1               ; $F2A7: 90 08    ; 2/3 cycles
; Conditional paths join or continue at L_F2B1 and L_F2AB.
L_F2A9:
        beq    L_F2B1               ; $F2A9: F0 06    ; 2/3 cycles
; Continue at L_F2A5.
L_F2AB:
        iny                         ; $F2AB: C8
        iny                         ; $F2AC: C8
        iny                         ; $F2AD: C8
        jmp    L_F2A5               ; $F2AE: 4C A5 F2
; Return through the matching JSR continuation.
L_F2B1:
        rts                         ; $F2B1: 60

; -----------------------------------------------------------------------------
; Update the inactivity counter $E5/$E6; joystick or switch activity clears its high byte.
; JSR sites: $F30C.
L_F2B2:
        inc.b  $E5                  ; $F2B2: E6 E5
        bne    L_F2BE               ; $F2B4: D0 08    ; 2/3 cycles
; Conditional paths join or continue at L_F2BE and L_F2BA.
L_F2B6:
        inc.b  $E6                  ; $F2B6: E6 E6
        bne    L_F2BE               ; $F2B8: D0 04    ; 2/3 cycles
; Fall through to L_F2BE.
L_F2BA:
        lda    #$80                 ; $F2BA: A9 80
        sta.b  $E6                  ; $F2BC: 85 E6
; Conditional paths join or continue at L_F2CE and L_F2C5.
L_F2BE:
        lda.w  SWCHA                ; $F2BE: AD 80 02
        cmp    #$FF                 ; $F2C1: C9 FF
        bne    L_F2CE               ; $F2C3: D0 09    ; 2/3 cycles
; Conditional paths join or continue at L_F2D2 and L_F2CE.
L_F2C5:
        lda.w  SWCHB                ; $F2C5: AD 82 02
        and    #$03                 ; $F2C8: 29 03
        cmp    #$03                 ; $F2CA: C9 03
        beq    L_F2D2               ; $F2CC: F0 04    ; 2/3 cycles
; Fall through to L_F2D2.
L_F2CE:
        lda    #$00                 ; $F2CE: A9 00
        sta.b  $E6                  ; $F2D0: 85 E6
; Return through the matching JSR continuation.
L_F2D2:
        rts                         ; $F2D2: 60

; -----------------------------------------------------------------------------
; Resolve a color byte, optionally through indexed RAM, and apply the inactivity color transform.
; JSR sites: $F12B, $F137, $F13E, $F1BD, $F1C8, $F21D, $F228.
L_F2D3:
        lsr                         ; $F2D3: 4A
        bcc    L_F2DA               ; $F2D4: 90 04    ; 2/3 cycles
; Fall through to L_F2DA.
L_F2D6:
        tay                         ; $F2D6: A8
        lda.w  $0080,y              ; $F2D7: B9 80 00
; Conditional paths join or continue at L_F2E2 and L_F2DE.
L_F2DA:
        ldy.b  $E6                  ; $F2DA: A4 E6
        bpl    L_F2E2               ; $F2DC: 10 04    ; 2/3 cycles
; Fall through to L_F2E2.
L_F2DE:
        eor.b  $E6                  ; $F2DE: 45 E6
        and    #$FB                 ; $F2E0: 29 FB
; Return through the matching JSR continuation.
L_F2E2:
        asl                         ; $F2E2: 0A
        rts                         ; $F2E3: 60

; -----------------------------------------------------------------------------
; Load $93/$94 from the position pointer in the descriptor at $FF44+X.
; JSR sites: $F595, $F5DA, $F972.
L_F2E4:
        lda.w  D_FF44,x             ; $F2E4: BD 44 FF
        sta.b  $93                  ; $F2E7: 85 93
        lda.w  D_FF45,x             ; $F2E9: BD 45 FF
        sta.b  $94                  ; $F2EC: 85 94
        rts                         ; $F2EE: 60

; -----------------------------------------------------------------------------
; Initialize the TIA, set SP to $FF, and clear RAM before entering the scheduler.
L_F2EF:
        sei                         ; $F2EF: 78
        cld                         ; $F2F0: D8
        ldx    #$28                 ; $F2F1: A2 28
        lda    #$00                 ; $F2F3: A9 00
; Clear TIA write registers $04-$2C.
L_F2F5:
        sta.b  NUSIZ0,x             ; $F2F5: 95 04
        dex                         ; $F2F7: CA
        bpl    L_F2F5               ; $F2F8: 10 FB    ; 2/3 cycles
; X is $FF after the TIA loop; initialize SP once. This TXS does not redirect control flow.
L_F2FA:
        txs                         ; $F2FA: 9A
; Clear zero-page RAM $80-$FF with zero-page indexed stores.
L_F2FB:
        sta.b  VSYNC,x              ; $F2FB: 95 00
        dex                         ; $F2FD: CA
        bmi    L_F2FB               ; $F2FE: 30 FB    ; 2/3 cycles (+1 if taken across page)
; Call L_F371; continue at L_F303.
L_F300:
        jsr    L_F371               ; $F300: 20 71 F3
; Call L_F3D3; continue at L_F306.
L_F303:
        jsr    L_F3D3               ; $F303: 20 D3 F3
; MAIN SCHEDULER: poll switches, advance effects, and update inactivity before the frame phases.
L_F306:
        jsr    L_F384               ; $F306: 20 84 F3
; Call L_FA23; continue at L_F30C.
L_F309:
        jsr    L_FA23               ; $F309: 20 23 FA
; Call L_F2B2; continue at L_F30F.
L_F30C:
        jsr    L_F2B2               ; $F30C: 20 B2 F2
; Conditional paths join or continue at L_F365 and L_F313.
L_F30F:
        lda.b  $DE                  ; $F30F: A5 DE
        bne    L_F365               ; $F311: D0 52    ; 2/3 cycles
; Conditional paths join or continue at L_F323 and L_F319.
L_F313:
        lda.b  $B9                  ; $F313: A5 B9
        cmp    #$12                 ; $F315: C9 12
        bne    L_F323               ; $F317: D0 0A    ; 2/3 cycles
; Fall through to L_F323.
L_F319:
        lda    #$FF                 ; $F319: A9 FF
        sta.b  $DF                  ; $F31B: 85 DF
        sta.b  $DE                  ; $F31D: 85 DE
        lda    #$00                 ; $F31F: A9 00
        sta.b  $E0                  ; $F321: 85 E0
; PHASE 0: sample player input, update the held object, synchronize, prepare graphics, display, then handle contact.
L_F323:
        ldy    #$00                 ; $F323: A0 00
        jsr    L_F4C2               ; $F325: 20 C2 F4
; Call L_F5D4; continue at L_F32B.
L_F328:
        jsr    L_F5D4               ; $F328: 20 D4 F5
; Call L_F0EC; continue at L_F32E.
L_F32B:
        jsr    L_F0EC               ; $F32B: 20 EC F0
; Call L_F10F; continue at L_F331.
L_F32E:
        jsr    L_F10F               ; $F32E: 20 0F F1
; Call L_F008; continue at L_F334.
L_F331:
        jsr    L_F008               ; $F331: 20 08 F0
; Call L_F556; continue at L_F337.
L_F334:
        jsr    L_F556               ; $F334: 20 56 F5
; PHASE 1: update player motion and special display state; synchronize, update actors/gates, display.
L_F337:
        ldy    #$01                 ; $F337: A0 01
        jsr    L_F4C2               ; $F339: 20 C2 F4
; Call L_F9E7; continue at L_F33F.
L_F33C:
        jsr    L_F9E7               ; $F33C: 20 E7 F9
; Call L_F0EC; continue at L_F342.
L_F33F:
        jsr    L_F0EC               ; $F33F: 20 EC F0
; Call L_F8A5; continue at L_F345.
L_F342:
        jsr    L_F8A5               ; $F342: 20 A5 F8
; Call L_F93C; continue at L_F348.
L_F345:
        jsr    L_F93C               ; $F345: 20 3C F9
; Call L_F008; continue at L_F34B.
L_F348:
        jsr    L_F008               ; $F348: 20 08 F0
; Overscan work after phase 1: update two actors, then synchronize for phase 2.
L_F34B:
        jsr    L_F7CB               ; $F34B: 20 CB F7
; Call L_F7B0; continue at L_F351.
L_F34E:
        jsr    L_F7B0               ; $F34E: 20 B0 F7
; Call L_F0EC; continue at L_F354.
L_F351:
        jsr    L_F0EC               ; $F351: 20 EC F0
; PHASE 2: update player motion and remaining actors, display, then repeat the scheduler.
L_F354:
        ldy    #$02                 ; $F354: A0 02
        jsr    L_F4C2               ; $F356: 20 C2 F4
; Call L_F795; continue at L_F35C.
L_F359:
        jsr    L_F795               ; $F359: 20 95 F7
; Call L_F9B3; continue at L_F35F.
L_F35C:
        jsr    L_F9B3               ; $F35C: 20 B3 F9
; Call L_F008; continue at L_F362.
L_F35F:
        jsr    L_F008               ; $F35F: 20 08 F0
; Continue at L_F306.
L_F362:
        jmp    L_F306               ; $F362: 4C 06 F3
; Alternate scheduler path: synchronize, display the retained state, then refresh its graphics setup.
L_F365:
        jsr    L_F0EC               ; $F365: 20 EC F0
; Call L_F008; continue at L_F36B.
L_F368:
        jsr    L_F008               ; $F368: 20 08 F0
; Call L_F10F; continue at L_F36E.
L_F36B:
        jsr    L_F10F               ; $F36B: 20 0F F1
; Continue at L_F306.
L_F36E:
        jmp    L_F306               ; $F36E: 4C 06 F3

; -----------------------------------------------------------------------------
; Position the two missiles with fixed coordinates, then apply their horizontal motion.
; JSR sites: $F300.
L_F371:
        lda    #$0D                 ; $F371: A9 0D
        ldx    #$02                 ; $F373: A2 02
        jsr    L_F0D2               ; $F375: 20 D2 F0
; Call L_F0D2; continue at L_F37F.
L_F378:
        lda    #$96                 ; $F378: A9 96
        ldx    #$03                 ; $F37A: A2 03
        jsr    L_F0D2               ; $F37C: 20 D2 F0
; Missile-positioning horizontal boundary; apply HMOVE immediately after the wait.
L_F37F:
        sta.b  WSYNC                ; $F37F: 85 02    ; horizontal wait
        sta.b  HMOVE                ; $F381: 85 2A
        rts                         ; $F383: 60

; -----------------------------------------------------------------------------
; Process newly pressed reset/select switches; share the initialization tail at $F3D3.
; JSR sites: $F306.
L_F384:
        lda.w  SWCHB                ; $F384: AD 82 02
        eor    #$FF                 ; $F387: 49 FF
        and.b  $92                  ; $F389: 25 92
        and    #$01                 ; $F38B: 29 01
        beq    L_F3B5               ; $F38D: F0 26    ; 2/3 cycles
; Conditional paths join or continue at L_F3D3 and L_F395.
L_F38F:
        lda.b  $DE                  ; $F38F: A5 DE
        cmp    #$FF                 ; $F391: C9 FF
        beq    L_F3D3               ; $F393: F0 3E    ; 2/3 cycles
; Fall through to L_F3B5.
L_F395:
        lda    #$11                 ; $F395: A9 11
        sta.b  $8A                  ; $F397: 85 8A
        sta.b  $E2                  ; $F399: 85 E2
        lda    #$50                 ; $F39B: A9 50
        sta.b  $8B                  ; $F39D: 85 8B
        sta.b  $E3                  ; $F39F: 85 E3
        lda    #$20                 ; $F3A1: A9 20
        sta.b  $8C                  ; $F3A3: 85 8C
        sta.b  $E4                  ; $F3A5: 85 E4
        lda    #$00                 ; $F3A7: A9 00
        sta.b  $A8                  ; $F3A9: 85 A8
        sta.b  $AD                  ; $F3AB: 85 AD
        sta.b  $B2                  ; $F3AD: 85 B2
        sta.b  $DF                  ; $F3AF: 85 DF
        lda    #$A2                 ; $F3B1: A9 A2
        sta.b  $9D                  ; $F3B3: 85 9D
; Conditional paths join or continue at L_F40C and L_F3C0.
L_F3B5:
        lda.w  SWCHB                ; $F3B5: AD 82 02
        eor    #$FF                 ; $F3B8: 49 FF
        and.b  $92                  ; $F3BA: 25 92
        and    #$02                 ; $F3BC: 29 02
        beq    L_F40C               ; $F3BE: F0 4C    ; 2/3 cycles (+1 if taken across page)
; Conditional paths join or continue at L_F3D3 and L_F3C6.
L_F3C0:
        lda.b  $8A                  ; $F3C0: A5 8A
        cmp    #$00                 ; $F3C2: C9 00
        bne    L_F3D3               ; $F3C4: D0 0D    ; 2/3 cycles
; Conditional paths join or continue at L_F3D1 and L_F3CF.
L_F3C6:
        lda.b  $DD                  ; $F3C6: A5 DD
        clc                         ; $F3C8: 18
        adc    #$02                 ; $F3C9: 69 02
        cmp    #$06                 ; $F3CB: C9 06
        bcc    L_F3D1               ; $F3CD: 90 02    ; 2/3 cycles
; Fall through to L_F3D1.
L_F3CF:
        lda    #$00                 ; $F3CF: A9 00
; Fall through to L_F3D3.
L_F3D1:
        sta.b  $DD                  ; $F3D1: 85 DD

; -----------------------------------------------------------------------------
; Initialize a selected game from the 49-byte ROM image; optionally randomize object placement.
; JSR sites: $F303.
L_F3D3:
        lda    #$00                 ; $F3D3: A9 00
        sta.b  $8A                  ; $F3D5: 85 8A
        sta.b  $E2                  ; $F3D7: 85 E2
        lda    #$00                 ; $F3D9: A9 00
        sta.b  $8C                  ; $F3DB: 85 8C
        sta.b  $E4                  ; $F3DD: 85 E4
        ldy.b  $DD                  ; $F3DF: A4 DD
        lda.w  D_F45A,y             ; $F3E1: B9 5A F4
        sta.b  $93                  ; $F3E4: 85 93
        lda.w  D_F45B,y             ; $F3E6: B9 5B F4
        sta.b  $94                  ; $F3E9: 85 94
        ldy    #$30                 ; $F3EB: A0 30
; Copy the selected initial-state image backward into RAM $A1-$D1 (49 bytes).
L_F3ED:
        lda    ($93),y              ; $F3ED: B1 93
        sta.w  $00A1,y              ; $F3EF: 99 A1 00
        dey                         ; $F3F2: 88
        bpl    L_F3ED               ; $F3F3: 10 F8    ; 2/3 cycles
; Conditional paths join or continue at L_F404 and L_F3FB.
L_F3F5:
        lda.b  $DD                  ; $F3F5: A5 DD
        cmp    #$04                 ; $F3F7: C9 04
        bcc    L_F404               ; $F3F9: 90 09    ; 2/3 cycles (+1 if taken across page)
; Call L_F412; continue at L_F3FE.
L_F3FB:
        jsr    L_F412               ; $F3FB: 20 12 F4
; Call L_F0EC; continue at L_F401.
L_F3FE:
        jsr    L_F0EC               ; $F3FE: 20 EC F0
; Call L_F008; continue at L_F404.
L_F401:
        jsr    L_F008               ; $F401: 20 08 F0
; Fall through to L_F40C.
L_F404:
        lda    #$00                 ; $F404: A9 00
        sta.b  $DE                  ; $F406: 85 DE
        lda    #$A2                 ; $F408: A9 A2
        sta.b  $9D                  ; $F40A: 85 9D
; Return through the matching JSR continuation.
L_F40C:
        lda.w  SWCHB                ; $F40C: AD 82 02
        sta.b  $92                  ; $F40F: 85 92
        rts                         ; $F411: 60

; -----------------------------------------------------------------------------
; Generate constrained room placements from $E5 and the triples at $F439.
; JSR sites: $F3FB.
L_F412:
        ldy    #$1E                 ; $F412: A0 1E
; Conditional paths join or continue at L_F414 and L_F427.
L_F414:
        lda.b  $E5                  ; $F414: A5 E5
        lsr                         ; $F416: 4A
        lsr                         ; $F417: 4A
        lsr                         ; $F418: 4A
        lsr                         ; $F419: 4A
        lsr                         ; $F41A: 4A
        sec                         ; $F41B: 38
        adc.b  $E5                  ; $F41C: 65 E5
        sta.b  $E5                  ; $F41E: 85 E5
        and    #$1F                 ; $F420: 29 1F
        cmp.w  D_F43A,y             ; $F422: D9 3A F4
        bcc    L_F414               ; $F425: 90 ED    ; 2/3 cycles
; Conditional paths join or continue at L_F42E and L_F42C.
L_F427:
        cmp.w  D_F43B,y             ; $F427: D9 3B F4
        beq    L_F42E               ; $F42A: F0 02    ; 2/3 cycles
; Conditional paths join or continue at L_F414 and L_F42E.
L_F42C:
        bcs    L_F414               ; $F42C: B0 E6    ; 2/3 cycles
; Conditional paths join or continue at L_F414 and L_F438.
L_F42E:
        ldx.w  D_F439,y             ; $F42E: BE 39 F4
        sta.b  VSYNC,x              ; $F431: 95 00
        dey                         ; $F433: 88
        dey                         ; $F434: 88
        dey                         ; $F435: 88
        bpl    L_F414               ; $F436: 10 DC    ; 2/3 cycles
; Return through the matching JSR continuation.
L_F438:
        rts                         ; $F438: 60

; Placement constraints: eleven triples (RAM address, minimum room, maximum room).
D_F439:
        .byte $B9                                              ; $F439-$F439

; Interior table address read directly by code.
D_F43A:
        .byte $13                                              ; $F43A-$F43A

; Interior table address read directly by code.
D_F43B:
        .byte $1A,$A4,$01,$1D,$A9,$01,$1D,$AE,$01,$1D,$B6,$01  ; $F43B-$F446
        .byte $1D,$BC,$01,$1D,$BF,$01,$1D,$C2,$01,$16,$C5,$01  ; $F447-$F452
        .byte $12,$CB,$01,$1D,$B3,$01,$1D                      ; $F453-$F459

; Initial-state pointers for the three game selections; selections 1 and 2 share $F491.
D_F45A:
        .byte $60                                              ; $F45A-$F45A

; Interior table address read directly by code.
D_F45B:
        .byte $F4,$91,$F4,$91,$F4                              ; $F45B-$F45F

; 49-byte initial RAM image, copied to $A1-$D1 for game selection 0.
D_F460:
        .byte $15,$51,$12,$0E,$50,$20,$00,$00,$01,$50,$20,$00  ; $F460-$F46B
        .byte $00,$1D,$50,$20,$00,$00,$1B,$80,$20,$12,$20,$20  ; $F46C-$F477
        .byte $1C,$30,$20,$04,$29,$37,$11,$20,$40,$0E,$20,$40  ; $F478-$F483
        .byte $1D,$20,$40,$1C,$1C,$1C,$1A,$20,$20,$00,$00,$78  ; $F484-$F48F
        .byte $00                                              ; $F490-$F490

; 49-byte initial RAM image, copied to $A1-$D1 for game selections 1 and 2.
D_F491:
        .byte $15,$51,$12,$14,$50,$20,$A0,$00,$19,$50,$20,$A0  ; $F491-$F49C
        .byte $00,$04,$50,$20,$A0,$00,$0E,$80,$20,$11,$20,$20  ; $F49D-$F4A8
        .byte $14,$30,$20,$0B,$40,$40,$09,$20,$40,$06,$20,$40  ; $F4A9-$F4B4
        .byte $19,$20,$40,$1C,$1C,$1C,$02,$20,$20,$90,$00,$78  ; $F4B5-$F4C0
        .byte $00                                              ; $F4C1-$F4C1

; -----------------------------------------------------------------------------
; Resolve collision feedback and joystick phase masks; move the player and retain its saved coordinates.
; JSR sites: $F325, $F339, $F356.
L_F4C2:
        lda.b  CXBLPF               ; $F4C2: A5 36
        and    #$80                 ; $F4C4: 29 80
        bne    L_F4F5               ; $F4C6: D0 2D    ; 2/3 cycles
; Conditional paths join or continue at L_F4F5 and L_F4CE.
L_F4C8:
        lda.b  CXM0FB               ; $F4C8: A5 34
        and    #$40                 ; $F4CA: 29 40
        bne    L_F4F5               ; $F4CC: D0 27    ; 2/3 cycles
; Conditional paths join or continue at L_F4DA and L_F4D4.
L_F4CE:
        lda.b  CXM1FB               ; $F4CE: A5 35
        and    #$40                 ; $F4D0: 29 40
        beq    L_F4DA               ; $F4D2: F0 06    ; 2/3 cycles
; Conditional paths join or continue at L_F4F5 and L_F4DA.
L_F4D4:
        lda.b  $96                  ; $F4D4: A5 96
        cmp    #$87                 ; $F4D6: C9 87
        bne    L_F4F5               ; $F4D8: D0 1B    ; 2/3 cycles
; Conditional paths join or continue at L_F4E6 and L_F4E0.
L_F4DA:
        lda.b  CXP0FB               ; $F4DA: A5 32
        and    #$40                 ; $F4DC: 29 40
        beq    L_F4E6               ; $F4DE: F0 06    ; 2/3 cycles
; Conditional paths join or continue at L_F4F5 and L_F4E6.
L_F4E0:
        lda.b  $95                  ; $F4E0: A5 95
        cmp    #$00                 ; $F4E2: C9 00
        bne    L_F4F5               ; $F4E4: D0 0F    ; 2/3 cycles
; Conditional paths join or continue at L_F51F and L_F4EC.
L_F4E6:
        lda.b  CXP1FB               ; $F4E6: A5 33
        and    #$40                 ; $F4E8: 29 40
        beq    L_F51F               ; $F4EA: F0 33    ; 2/3 cycles (+1 if taken across page)
; Conditional paths join or continue at L_F4F5 and L_F4F2.
L_F4EC:
        lda.b  $96                  ; $F4EC: A5 96
        cmp    #$00                 ; $F4EE: C9 00
        bne    L_F4F5               ; $F4F0: D0 03    ; 2/3 cycles
; Continue at L_F51F.
L_F4F2:
        jmp    L_F51F               ; $F4F2: 4C 1F F5
; Conditional paths join or continue at L_F52F and L_F4F9.
L_F4F5:
        cpy    #$02                 ; $F4F5: C0 02
        bne    L_F52F               ; $F4F7: D0 36    ; 2/3 cycles (+1 if taken across page)
; Conditional paths join or continue at L_F52F and L_F4FF.
L_F4F9:
        lda.b  $9D                  ; $F4F9: A5 9D
        cmp    #$5A                 ; $F4FB: C9 5A
        beq    L_F52F               ; $F4FD: F0 30    ; 2/3 cycles (+1 if taken across page)
; Conditional paths join or continue at L_F52F and L_F505.
L_F4FF:
        lda.b  $8A                  ; $F4FF: A5 8A
        cmp.b  $BC                  ; $F501: C5 BC
        bne    L_F52F               ; $F503: D0 2A    ; 2/3 cycles
; Conditional paths join or continue at L_F52F and L_F50E.
L_F505:
        lda.b  $8B                  ; $F505: A5 8B
        sec                         ; $F507: 38
        sbc.b  $BD                  ; $F508: E5 BD
        cmp    #$0A                 ; $F50A: C9 0A
        bcc    L_F52F               ; $F50C: 90 21    ; 2/3 cycles
; Conditional paths join or continue at L_F52F and L_F512.
L_F50E:
        cmp    #$17                 ; $F50E: C9 17
        bcs    L_F52F               ; $F510: B0 1D    ; 2/3 cycles
; Conditional paths join or continue at L_F51F and L_F51B.
L_F512:
        lda.b  $BE                  ; $F512: A5 BE
        sec                         ; $F514: 38
        sbc.b  $8C                  ; $F515: E5 8C
        cmp    #$FC                 ; $F517: C9 FC
        bcs    L_F51F               ; $F519: B0 04    ; 2/3 cycles
; Conditional paths join or continue at L_F52F and L_F51F.
L_F51B:
        cmp    #$19                 ; $F51B: C9 19
        bcs    L_F52F               ; $F51D: B0 10    ; 2/3 cycles
; Fall through to L_F52F.
L_F51F:
        lda    #$FF                 ; $F51F: A9 FF
        sta.b  $99                  ; $F521: 85 99
        lda.b  $8A                  ; $F523: A5 8A
        sta.b  $E2                  ; $F525: 85 E2
        lda.b  $8B                  ; $F527: A5 8B
        sta.b  $E3                  ; $F529: 85 E3
        lda.b  $8C                  ; $F52B: A5 8C
        sta.b  $E4                  ; $F52D: 85 E4
; Conditional paths join or continue at L_F538 and L_F533.
L_F52F:
        cpy    #$00                 ; $F52F: C0 00
        bne    L_F538               ; $F531: D0 05    ; 2/3 cycles
; Fall through to L_F538.
L_F533:
        lda.w  SWCHA                ; $F533: AD 80 02
        sta.b  $99                  ; $F536: 85 99
; Call L_F5FF; continue at L_F552.
L_F538:
        lda.b  $E2                  ; $F538: A5 E2
        sta.b  $8A                  ; $F53A: 85 8A
        lda.b  $E3                  ; $F53C: A5 E3
        sta.b  $8B                  ; $F53E: 85 8B
        lda.b  $E4                  ; $F540: A5 E4
        sta.b  $8C                  ; $F542: 85 8C
        lda.b  $99                  ; $F544: A5 99
        ora.w  D_F553,y             ; $F546: 19 53 F5
        sta.b  $9B                  ; $F549: 85 9B
        ldy    #$03                 ; $F54B: A0 03
        ldx    #$8A                 ; $F54D: A2 8A
        jsr    L_F5FF               ; $F54F: 20 FF F5
; Return through the matching JSR continuation.
L_F552:
        rts                         ; $F552: 60

; Three direction masks indexed by the scheduler phase Y = 0, 1, 2.
D_F553:
        .byte $00,$C0,$30                                      ; $F553-$F555

; -----------------------------------------------------------------------------
; Handle a new fire-button press and player/object contact; record a held descriptor and its relative position.
; JSR sites: $F334.
L_F556:
        rol.b  INPT4                ; $F556: 26 3C
        ror.b  $D7                  ; $F558: 66 D7
        lda.b  $D7                  ; $F55A: A5 D7
        and    #$C0                 ; $F55C: 29 C0
        cmp    #$40                 ; $F55E: C9 40
        bne    L_F572               ; $F560: D0 10    ; 2/3 cycles
; Conditional paths join or continue at L_F572 and L_F568.
L_F562:
        lda    #$A2                 ; $F562: A9 A2
        cmp.b  $9D                  ; $F564: C5 9D
        beq    L_F572               ; $F566: F0 0A    ; 2/3 cycles
; Fall through to L_F572.
L_F568:
        sta.b  $9D                  ; $F568: 85 9D
        lda    #$04                 ; $F56A: A9 04
        sta.b  $E0                  ; $F56C: 85 E0
        lda    #$04                 ; $F56E: A9 04
        sta.b  $DF                  ; $F570: 85 DF
; Conditional paths join or continue at L_F583 and L_F57C.
L_F572:
        lda    #$FF                 ; $F572: A9 FF
        sta.b  $98                  ; $F574: 85 98
        lda.b  CXP0FB               ; $F576: A5 32
        and    #$40                 ; $F578: 29 40
        beq    L_F583               ; $F57A: F0 07    ; 2/3 cycles
; Continue at L_F593.
L_F57C:
        lda.b  $95                  ; $F57C: A5 95
        sta.b  $97                  ; $F57E: 85 97
        jmp    L_F593               ; $F580: 4C 93 F5
; Conditional paths join or continue at L_F590 and L_F589.
L_F583:
        lda.b  CXP1FB               ; $F583: A5 33
        and    #$40                 ; $F585: 29 40
        beq    L_F590               ; $F587: F0 07    ; 2/3 cycles
; Continue at L_F593.
L_F589:
        lda.b  $96                  ; $F589: A5 96
        sta.b  $97                  ; $F58B: 85 97
        jmp    L_F593               ; $F58D: 4C 93 F5
; Continue at L_F5D3.
L_F590:
        jmp    L_F5D3               ; $F590: 4C D3 F5
; Call L_F2E4; continue at L_F598.
L_F593:
        ldx.b  $97                  ; $F593: A6 97
        jsr    L_F2E4               ; $F595: 20 E4 F2
; Conditional paths join or continue at L_F5D3 and L_F59E.
L_F598:
        lda.b  $97                  ; $F598: A5 97
        cmp    #$51                 ; $F59A: C9 51
        bcc    L_F5D3               ; $F59C: 90 35    ; 2/3 cycles
; Conditional paths join or continue at L_F5D3 and L_F5A6.
L_F59E:
        ldy    #$00                 ; $F59E: A0 00
        lda    ($93),y              ; $F5A0: B1 93
        cmp.b  $8A                  ; $F5A2: C5 8A
        bne    L_F5D3               ; $F5A4: D0 2D    ; 2/3 cycles
; Conditional paths join or continue at L_F5B4 and L_F5AC.
L_F5A6:
        lda.b  $97                  ; $F5A6: A5 97
        cmp.b  $9D                  ; $F5A8: C5 9D
        beq    L_F5B4               ; $F5AA: F0 08    ; 2/3 cycles
; Fall through to L_F5B4.
L_F5AC:
        lda    #$05                 ; $F5AC: A9 05
        sta.b  $E0                  ; $F5AE: 85 E0
        lda    #$04                 ; $F5B0: A9 04
        sta.b  $DF                  ; $F5B2: 85 DF
; Call L_F6AC; continue at L_F5C1.
L_F5B4:
        lda.b  $97                  ; $F5B4: A5 97
        sta.b  $9D                  ; $F5B6: 85 9D
        ldx.b  $93                  ; $F5B8: A6 93
        ldy    #$06                 ; $F5BA: A0 06
        lda.b  $99                  ; $F5BC: A5 99
        jsr    L_F6AC               ; $F5BE: 20 AC F6
; Fall through to L_F5D3.
L_F5C1:
        ldy    #$01                 ; $F5C1: A0 01
        lda    ($93),y              ; $F5C3: B1 93
        sec                         ; $F5C5: 38
        sbc.b  $8B                  ; $F5C6: E5 8B
        sta.b  $9E                  ; $F5C8: 85 9E
        ldy    #$02                 ; $F5CA: A0 02
        lda    ($93),y              ; $F5CC: B1 93
        sec                         ; $F5CE: 38
        sbc.b  $8C                  ; $F5CF: E5 8C
        sta.b  $9F                  ; $F5D1: 85 9F
; Return through the matching JSR continuation.
L_F5D3:
        rts                         ; $F5D3: 60

; -----------------------------------------------------------------------------
; Update the held object from the player coordinates and stored relative offsets, then resolve movement.
; JSR sites: $F328.
L_F5D4:
        ldx.b  $9D                  ; $F5D4: A6 9D
        cpx    #$A2                 ; $F5D6: E0 A2
        beq    L_F5FE               ; $F5D8: F0 24    ; 2/3 cycles
; Call L_F2E4; continue at L_F5DD.
L_F5DA:
        jsr    L_F2E4               ; $F5DA: 20 E4 F2
; Call L_F5FF; continue at L_F5FE.
L_F5DD:
        ldy    #$00                 ; $F5DD: A0 00
        lda.b  $8A                  ; $F5DF: A5 8A
        sta    ($93),y              ; $F5E1: 91 93
        ldy    #$01                 ; $F5E3: A0 01
        lda.b  $8B                  ; $F5E5: A5 8B
        clc                         ; $F5E7: 18
        adc.b  $9E                  ; $F5E8: 65 9E
        sta    ($93),y              ; $F5EA: 91 93
        ldy    #$02                 ; $F5EC: A0 02
        lda.b  $8C                  ; $F5EE: A5 8C
        clc                         ; $F5F0: 18
        adc.b  $9F                  ; $F5F1: 65 9F
        sta    ($93),y              ; $F5F3: 91 93
        ldy    #$00                 ; $F5F5: A0 00
        lda    #$FF                 ; $F5F7: A9 FF
        ldx.b  $93                  ; $F5F9: A6 93
        jsr    L_F5FF               ; $F5FB: 20 FF F5
; Return through the matching JSR continuation.
L_F5FE:
        rts                         ; $F5FE: 60

; -----------------------------------------------------------------------------
; Apply directional movement to the record at zero-page X; resolve gates and room-edge transitions.
; JSR sites: $F54F, $F5FB, $F724, $F8BD, $F9CF.
L_F5FF:
        jsr    L_F6AC               ; $F5FF: 20 AC F6
; Fall through to L_F604.
L_F602:
        ldy    #$02                 ; $F602: A0 02
; Conditional paths join or continue at L_F62F and L_F60D.
L_F604:
        sty.b  $9A                  ; $F604: 84 9A
        lda.w  $00C8,y              ; $F606: B9 C8 00
        cmp    #$1C                 ; $F609: C9 1C
        beq    L_F62F               ; $F60B: F0 22    ; 2/3 cycles
; Conditional paths join or continue at L_F62F and L_F616.
L_F60D:
        ldy.b  $9A                  ; $F60D: A4 9A
        lda.b  $00,x                ; $F60F: B5 00
        cmp.w  D_F9AD,y             ; $F611: D9 AD F9
        bne    L_F62F               ; $F614: D0 19    ; 2/3 cycles
; Conditional paths join or continue at L_F62F and L_F61C.
L_F616:
        lda.b  $02,x                ; $F616: B5 02
        cmp    #$0D                 ; $F618: C9 0D
        bpl    L_F62F               ; $F61A: 10 13    ; 2/3 cycles
; Return through the matching JSR continuation.
L_F61C:
        lda.w  D_F9B0,y             ; $F61C: B9 B0 F9
        sta.b  VSYNC,x              ; $F61F: 95 00
        lda    #$50                 ; $F621: A9 50
        sta.b  VBLANK,x             ; $F623: 95 01
        lda    #$2C                 ; $F625: A9 2C
        sta.b  WSYNC,x              ; $F627: 95 02    ; horizontal wait
        lda    #$01                 ; $F629: A9 01
        sta.w  $00C8,y              ; $F62B: 99 C8 00
        rts                         ; $F62E: 60
; Conditional paths join or continue at L_F604 and L_F634.
L_F62F:
        ldy.b  $9A                  ; $F62F: A4 9A
        dey                         ; $F631: 88
        bpl    L_F604               ; $F632: 10 D0    ; 2/3 cycles
; Conditional paths join or continue at L_F643 and L_F63A.
L_F634:
        lda.b  $02,x                ; $F634: B5 02
        cmp    #$6A                 ; $F636: C9 6A
        bmi    L_F643               ; $F638: 30 09    ; 2/3 cycles
; Continue at L_F69F.
L_F63A:
        lda    #$0D                 ; $F63A: A9 0D
        sta.b  WSYNC,x              ; $F63C: 95 02    ; horizontal wait
        ldy    #$05                 ; $F63E: A0 05
        jmp    L_F69F               ; $F640: 4C 9F F6
; Conditional paths join or continue at L_F650 and L_F649.
L_F643:
        lda.b  $01,x                ; $F643: B5 01
        cmp    #$03                 ; $F645: C9 03
        bcc    L_F650               ; $F647: 90 07    ; 2/3 cycles
; Conditional paths join or continue at L_F650 and L_F64D.
L_F649:
        cmp    #$F0                 ; $F649: C9 F0
        bcs    L_F650               ; $F64B: B0 03    ; 2/3 cycles
; Continue at L_F662.
L_F64D:
        jmp    L_F662               ; $F64D: 4C 62 F6
; Conditional paths join or continue at L_F659 and L_F654.
L_F650:
        cpx    #$8A                 ; $F650: E0 8A
        beq    L_F659               ; $F652: F0 05    ; 2/3 cycles
; Continue at L_F65B.
L_F654:
        lda    #$9A                 ; $F654: A9 9A
        jmp    L_F65B               ; $F656: 4C 5B F6
; Fall through to L_F65B.
L_F659:
        lda    #$9E                 ; $F659: A9 9E
; Continue at L_F69F.
L_F65B:
        sta.b  VBLANK,x             ; $F65B: 95 01
        ldy    #$08                 ; $F65D: A0 08
        jmp    L_F69F               ; $F65F: 4C 9F F6
; Conditional paths join or continue at L_F671 and L_F668.
L_F662:
        lda.b  $02,x                ; $F662: B5 02
        cmp    #$0D                 ; $F664: C9 0D
        bcs    L_F671               ; $F666: B0 09    ; 2/3 cycles
; Continue at L_F69F.
L_F668:
        lda    #$69                 ; $F668: A9 69
        sta.b  WSYNC,x              ; $F66A: 95 02    ; horizontal wait
        ldy    #$07                 ; $F66C: A0 07
        jmp    L_F69F               ; $F66E: 4C 9F F6
; Conditional paths join or continue at L_F692 and L_F677.
L_F671:
        lda.b  $01,x                ; $F671: B5 01
        cpx    #$8A                 ; $F673: E0 8A
        bne    L_F692               ; $F675: D0 1B    ; 2/3 cycles
; Conditional paths join or continue at L_F6AB and L_F67B.
L_F677:
        cmp    #$9F                 ; $F677: C9 9F
        bcc    L_F6AB               ; $F679: 90 30    ; 2/3 cycles
; Conditional paths join or continue at L_F696 and L_F681.
L_F67B:
        lda.b  $00,x                ; $F67B: B5 00
        cmp    #$03                 ; $F67D: C9 03
        bne    L_F696               ; $F67F: D0 15    ; 2/3 cycles
; Conditional paths join or continue at L_F696 and L_F687.
L_F681:
        lda.b  $A1                  ; $F681: A5 A1
        cmp    #$15                 ; $F683: C9 15
        beq    L_F696               ; $F685: F0 0F    ; 2/3 cycles
; Continue at L_F6AB.
L_F687:
        lda    #$1E                 ; $F687: A9 1E
        sta.b  VSYNC,x              ; $F689: 95 00
        lda    #$03                 ; $F68B: A9 03
        sta.b  VBLANK,x             ; $F68D: 95 01
        jmp    L_F6AB               ; $F68F: 4C AB F6
; Conditional paths join or continue at L_F6AB and L_F696.
L_F692:
        cmp    #$9B                 ; $F692: C9 9B
        bcc    L_F6AB               ; $F694: 90 15    ; 2/3 cycles
; Continue at L_F69F.
L_F696:
        lda    #$03                 ; $F696: A9 03
        sta.b  VBLANK,x             ; $F698: 95 01
        ldy    #$06                 ; $F69A: A0 06
        jmp    L_F69F               ; $F69C: 4C 9F F6
; Read the room exit selected by Y, resolve tagged destinations, and replace the object room.
L_F69F:
        lda.b  $00,x                ; $F69F: B5 00
        jsr    L_F271               ; $F6A1: 20 71 F2
; Call L_F6D5; continue at L_F6A9.
L_F6A4:
        lda    ($93),y              ; $F6A4: B1 93
        jsr    L_F6D5               ; $F6A6: 20 D5 F6
; Fall through to L_F6AB.
L_F6A9:
        sta.b  VSYNC,x              ; $F6A9: 95 00
; Return through the matching JSR continuation.
L_F6AB:
        rts                         ; $F6AB: 60

; -----------------------------------------------------------------------------
; Move a room/x/y record by Y steps according to active-low direction bits in A.
; JSR sites: $F5BE, $F5FF.
L_F6AC:
        sta.b  $9B                  ; $F6AC: 85 9B
; Conditional paths join or continue at L_F6D4 and L_F6B1.
L_F6AE:
        dey                         ; $F6AE: 88
        bmi    L_F6D4               ; $F6AF: 30 23    ; 2/3 cycles
; Conditional paths join or continue at L_F6B9 and L_F6B7.
L_F6B1:
        lda.b  $9B                  ; $F6B1: A5 9B
        and    #$80                 ; $F6B3: 29 80
        bne    L_F6B9               ; $F6B5: D0 02    ; 2/3 cycles
; Fall through to L_F6B9.
L_F6B7:
        inc.b  $01,x                ; $F6B7: F6 01
; Conditional paths join or continue at L_F6C1 and L_F6BF.
L_F6B9:
        lda.b  $9B                  ; $F6B9: A5 9B
        and    #$40                 ; $F6BB: 29 40
        bne    L_F6C1               ; $F6BD: D0 02    ; 2/3 cycles
; Fall through to L_F6C1.
L_F6BF:
        dec.b  $01,x                ; $F6BF: D6 01
; Conditional paths join or continue at L_F6C9 and L_F6C7.
L_F6C1:
        lda.b  $9B                  ; $F6C1: A5 9B
        and    #$10                 ; $F6C3: 29 10
        bne    L_F6C9               ; $F6C5: D0 02    ; 2/3 cycles
; Fall through to L_F6C9.
L_F6C7:
        inc.b  $02,x                ; $F6C7: F6 02
; Conditional paths join or continue at L_F6D1 and L_F6CF.
L_F6C9:
        lda.b  $9B                  ; $F6C9: A5 9B
        and    #$20                 ; $F6CB: 29 20
        bne    L_F6D1               ; $F6CD: D0 02    ; 2/3 cycles
; Fall through to L_F6D1.
L_F6CF:
        dec.b  $02,x                ; $F6CF: D6 02
; Continue at L_F6AE.
L_F6D1:
        jmp    L_F6AE               ; $F6D1: 4C AE F6
; Return through the matching JSR continuation.
L_F6D4:
        rts                         ; $F6D4: 60

; -----------------------------------------------------------------------------
; Resolve tagged room exits through $FF32 and the selected game; leave direct room numbers unchanged.
; JSR sites: $F6A6.
L_F6D5:
        cmp    #$80                 ; $F6D5: C9 80
        bcc    L_F6E8               ; $F6D7: 90 0F    ; 2/3 cycles
; Fall through to L_F6E8.
L_F6D9:
        sec                         ; $F6D9: 38
        sbc    #$80                 ; $F6DA: E9 80
        sta.b  $D8                  ; $F6DC: 85 D8
        lda.b  $DD                  ; $F6DE: A5 DD
        lsr                         ; $F6E0: 4A
        clc                         ; $F6E1: 18
        adc.b  $D8                  ; $F6E2: 65 D8
        tay                         ; $F6E4: A8
        lda.w  D_FF32,y             ; $F6E5: B9 32 FF
; Return through the matching JSR continuation.
L_F6E8:
        rts                         ; $F6E8: 60

; -----------------------------------------------------------------------------
; Return the player collision bit for the descriptor in A if it is one of the displayed objects.
; JSR sites: $F80D, $F87B, $F95A.
L_F6E9:
        cmp.b  $95                  ; $F6E9: C5 95
        beq    L_F6F4               ; $F6EB: F0 07    ; 2/3 cycles
; Conditional paths join or continue at L_F6F9 and L_F6F1.
L_F6ED:
        cmp.b  $96                  ; $F6ED: C5 96
        beq    L_F6F9               ; $F6EF: F0 08    ; 2/3 cycles
; Return through the matching JSR continuation.
L_F6F1:
        lda    #$00                 ; $F6F1: A9 00
        rts                         ; $F6F3: 60
; Return through the matching JSR continuation.
L_F6F4:
        lda.b  CXP0FB               ; $F6F4: A5 32
        and    #$40                 ; $F6F6: 29 40
        rts                         ; $F6F8: 60
; Return through the matching JSR continuation.
L_F6F9:
        lda.b  CXP1FB               ; $F6F9: A5 33
        and    #$40                 ; $F6FB: 29 40
        rts                         ; $F6FD: 60

; -----------------------------------------------------------------------------
; Return the other displayed descriptor when the two player sprites collide; otherwise return sentinel $A2.
; JSR sites: $F836, $F941.
L_F6FE:
        lda.b  CXPPMM               ; $F6FE: A5 37
        and    #$80                 ; $F700: 29 80
        beq    L_F70C               ; $F702: F0 08    ; 2/3 cycles
; Conditional paths join or continue at L_F70F and L_F708.
L_F704:
        cpx.b  $95                  ; $F704: E4 95
        beq    L_F70F               ; $F706: F0 07    ; 2/3 cycles
; Conditional paths join or continue at L_F712 and L_F70C.
L_F708:
        cpx.b  $96                  ; $F708: E4 96
        beq    L_F712               ; $F70A: F0 06    ; 2/3 cycles
; Return through the matching JSR continuation.
L_F70C:
        lda    #$A2                 ; $F70C: A9 A2
        rts                         ; $F70E: 60
; Return through the matching JSR continuation.
L_F70F:
        lda.b  $96                  ; $F70F: A5 96
        rts                         ; $F711: 60
; Return through the matching JSR continuation.
L_F712:
        lda.b  $95                  ; $F712: A5 95
        rts                         ; $F714: 60

; -----------------------------------------------------------------------------
; Find an actor target, preserve its previous direction if none is selected, and apply movement.
; JSR sites: $F808, $F8D7.
L_F715:
        jsr    L_F728               ; $F715: 20 28 F7
; Conditional paths join or continue at L_F720 and L_F71E.
L_F718:
        ldx.b  $D5                  ; $F718: A6 D5
        lda.b  $9B                  ; $F71A: A5 9B
        bne    L_F720               ; $F71C: D0 02    ; 2/3 cycles
; Fall through to L_F720.
L_F71E:
        lda.b  $03,x                ; $F71E: B5 03
; Call L_F5FF; continue at L_F727.
L_F720:
        sta.b  RSYNC,x              ; $F720: 95 03
        ldy.b  $D4                  ; $F722: A4 D4
        jsr    L_F5FF               ; $F724: 20 FF F5
; Return through the matching JSR continuation.
L_F727:
        rts                         ; $F727: 60

; -----------------------------------------------------------------------------
; Search ordered RAM-address pairs for two objects sharing a room, excluding the object at $D6.
; JSR sites: $F715, $F9C6.
; $D2/$D3 points to zero-terminated address pairs; $D5 is the moving record, $D4 the speed.
L_F728:
        lda    #$00                 ; $F728: A9 00
        sta.b  $E1                  ; $F72A: 85 E1
; Conditional paths join or continue at L_F748 and L_F73C.
L_F72C:
        ldy.b  $E1                  ; $F72C: A4 E1
        lda    ($D2),y              ; $F72E: B1 D2
        tax                         ; $F730: AA
        iny                         ; $F731: C8
        lda    ($D2),y              ; $F732: B1 D2
        tay                         ; $F734: A8
        lda.b  $00,x                ; $F735: B5 00
        cmp.w  $0000,y              ; $F737: D9 00 00
        bne    L_F748               ; $F73A: D0 0C    ; 2/3 cycles
; Conditional paths join or continue at L_F748 and L_F740.
L_F73C:
        cpy.b  $D6                  ; $F73C: C4 D6
        beq    L_F748               ; $F73E: F0 08    ; 2/3 cycles
; Conditional paths join or continue at L_F748 and L_F744.
L_F740:
        cpx.b  $D6                  ; $F740: E4 D6
        beq    L_F748               ; $F742: F0 04    ; 2/3 cycles
; Call L_F757; continue at L_F747.
L_F744:
        jsr    L_F757               ; $F744: 20 57 F7
; Return through the matching JSR continuation.
L_F747:
        rts                         ; $F747: 60
; Conditional paths join or continue at L_F72C and L_F752.
L_F748:
        inc.b  $E1                  ; $F748: E6 E1
        inc.b  $E1                  ; $F74A: E6 E1
        ldy.b  $E1                  ; $F74C: A4 E1
        lda    ($D2),y              ; $F74E: B1 D2
        bne    L_F72C               ; $F750: D0 DA    ; 2/3 cycles
; Return through the matching JSR continuation.
L_F752:
        lda    #$00                 ; $F752: A9 00
        sta.b  $9B                  ; $F754: 85 9B
        rts                         ; $F756: 60

; -----------------------------------------------------------------------------
; Compare two object positions and construct active-low directional bits in $9B.
; JSR sites: $F744.
L_F757:
        lda    #$FF                 ; $F757: A9 FF
        sta.b  $9B                  ; $F759: 85 9B
        lda.w  $0000,y              ; $F75B: B9 00 00
        cmp.b  $00,x                ; $F75E: D5 00
        bne    L_F792               ; $F760: D0 30    ; 2/3 cycles
; Conditional paths join or continue at L_F774 and L_F769.
L_F762:
        lda.w  $0001,y              ; $F762: B9 01 00
        cmp.b  $01,x                ; $F765: D5 01
        bcc    L_F774               ; $F767: 90 0B    ; 2/3 cycles
; Conditional paths join or continue at L_F77A and L_F76B.
L_F769:
        beq    L_F77A               ; $F769: F0 0F    ; 2/3 cycles
; Continue at L_F77A.
L_F76B:
        lda.b  $9B                  ; $F76B: A5 9B
        and    #$7F                 ; $F76D: 29 7F
        sta.b  $9B                  ; $F76F: 85 9B
        jmp    L_F77A               ; $F771: 4C 7A F7
; Fall through to L_F77A.
L_F774:
        lda.b  $9B                  ; $F774: A5 9B
        and    #$BF                 ; $F776: 29 BF
        sta.b  $9B                  ; $F778: 85 9B
; Conditional paths join or continue at L_F78C and L_F781.
L_F77A:
        lda.w  $0002,y              ; $F77A: B9 02 00
        cmp.b  $02,x                ; $F77D: D5 02
        bcc    L_F78C               ; $F77F: 90 0B    ; 2/3 cycles
; Conditional paths join or continue at L_F792 and L_F783.
L_F781:
        beq    L_F792               ; $F781: F0 0F    ; 2/3 cycles
; Continue at L_F792.
L_F783:
        lda.b  $9B                  ; $F783: A5 9B
        and    #$EF                 ; $F785: 29 EF
        sta.b  $9B                  ; $F787: 85 9B
        jmp    L_F792               ; $F789: 4C 92 F7
; Fall through to L_F792.
L_F78C:
        lda.b  $9B                  ; $F78C: A5 9B
        and    #$DF                 ; $F78E: 29 DF
        sta.b  $9B                  ; $F790: 85 9B
; Return through the matching JSR continuation.
L_F792:
        lda.b  $9B                  ; $F792: A5 9B
        rts                         ; $F794: 60

; -----------------------------------------------------------------------------
; Configure the first actor search list and movement speed, then call the shared actor update.
; JSR sites: $F359.
L_F795:
        lda    #<D_F7A7             ; $F795: A9 A7
        sta.b  $D2                  ; $F797: 85 D2
        lda    #>D_F7A7             ; $F799: A9 F7
        sta.b  $D3                  ; $F79B: 85 D3
        lda    #$03                 ; $F79D: A9 03
        sta.b  $D4                  ; $F79F: 85 D4
        ldx    #$36                 ; $F7A1: A2 36
        jsr    L_F7EA               ; $F7A3: 20 EA F7
; Return through the matching JSR continuation.
L_F7A6:
        rts                         ; $F7A6: 60

; Ordered RAM-address pairs for the first actor search; a zero byte terminates the list.
D_F7A7:
        .byte $B6,$A4,$A4,$8A,$A4,$B9,$A4,$C2,$00              ; $F7A7-$F7AF

; -----------------------------------------------------------------------------
; Configure the second actor search list and movement speed, then call the shared actor update.
; JSR sites: $F34E.
L_F7B0:
        lda    #<D_F7C2             ; $F7B0: A9 C2
        sta.b  $D2                  ; $F7B2: 85 D2
        lda    #>D_F7C2             ; $F7B4: A9 F7
        sta.b  $D3                  ; $F7B6: 85 D3
        lda    #$02                 ; $F7B8: A9 02
        sta.b  $D4                  ; $F7BA: 85 D4
        ldx    #$3F                 ; $F7BC: A2 3F
        jsr    L_F7EA               ; $F7BE: 20 EA F7
; Return through the matching JSR continuation.
L_F7C1:
        rts                         ; $F7C1: 60

; Ordered RAM-address pairs for the second actor search; a zero byte terminates the list.
D_F7C2:
        .byte $B6,$A9,$BF,$A9,$A9,$8A,$A9,$B9,$00              ; $F7C2-$F7CA

; -----------------------------------------------------------------------------
; Configure the third actor search list and movement speed, then call the shared actor update.
; JSR sites: $F34B.
L_F7CB:
        lda    #<D_F7DD             ; $F7CB: A9 DD
        sta.b  $D2                  ; $F7CD: 85 D2
        lda    #>D_F7DD             ; $F7CF: A9 F7
        sta.b  $D3                  ; $F7D1: 85 D3
        lda    #$02                 ; $F7D3: A9 02
        sta.b  $D4                  ; $F7D5: 85 D4
        ldx    #$48                 ; $F7D7: A2 48
        jsr    L_F7EA               ; $F7D9: 20 EA F7
; Return through the matching JSR continuation.
L_F7DC:
        rts                         ; $F7DC: 60

; Ordered RAM-address pairs for the third actor search; a zero byte terminates the list.
D_F7DD:
        .byte $B6,$AE,$AE,$8A,$AE,$B9,$AE,$BC,$AE,$B3,$AE,$C5  ; $F7DD-$F7E8
        .byte $00                                              ; $F7E9-$F7E9

; -----------------------------------------------------------------------------
; Update an actor state from target movement, difficulty, and the sprite collision latches.
; JSR sites: $F7A3, $F7BE, $F7D9.
L_F7EA:
        stx.b  $A0                  ; $F7EA: 86 A0
        lda.w  D_FF44,x             ; $F7EC: BD 44 FF
        tax                         ; $F7EF: AA
        lda.b  $04,x                ; $F7F0: B5 04
        cmp    #$00                 ; $F7F2: C9 00
        bne    L_F84E               ; $F7F4: D0 58    ; 2/3 cycles (+1 if taken across page)
; Conditional paths join or continue at L_F802 and L_F7FD.
L_F7F6:
        lda.w  SWCHB                ; $F7F6: AD 82 02
        and    #$80                 ; $F7F9: 29 80
        beq    L_F802               ; $F7FB: F0 05    ; 2/3 cycles (+1 if taken across page)
; Continue at L_F804.
L_F7FD:
        lda    #$00                 ; $F7FD: A9 00
        jmp    L_F804               ; $F7FF: 4C 04 F8
; Fall through to L_F804.
L_F802:
        lda    #$B6                 ; $F802: A9 B6
; Call L_F715; continue at L_F80B.
L_F804:
        sta.b  $D6                  ; $F804: 85 D6
        stx.b  $D5                  ; $F806: 86 D5
        jsr    L_F715               ; $F808: 20 15 F7
; Call L_F6E9; continue at L_F810.
L_F80B:
        lda.b  $A0                  ; $F80B: A5 A0
        jsr    L_F6E9               ; $F80D: 20 E9 F6
; Conditional paths join or continue at L_F832 and L_F812.
L_F810:
        beq    L_F832               ; $F810: F0 20    ; 2/3 cycles
; Fall through to L_F832.
L_F812:
        lda.w  SWCHB                ; $F812: AD 82 02
        rol                         ; $F815: 2A
        rol                         ; $F816: 2A
        rol                         ; $F817: 2A
        and    #$01                 ; $F818: 29 01
        ora.b  $DD                  ; $F81A: 05 DD
        tay                         ; $F81C: A8
        lda.w  D_F89F,y             ; $F81D: B9 9F F8
        sta.b  NUSIZ0,x             ; $F820: 95 04
        lda.b  $E3                  ; $F822: A5 E3
        sta.b  VBLANK,x             ; $F824: 95 01
        lda.b  $E4                  ; $F826: A5 E4
        sta.b  WSYNC,x              ; $F828: 95 02    ; horizontal wait
        lda    #$01                 ; $F82A: A9 01
        sta.b  $E0                  ; $F82C: 85 E0
        lda    #$10                 ; $F82E: A9 10
        sta.b  $DF                  ; $F830: 85 DF
; Call L_F6FE; continue at L_F839.
L_F832:
        stx.b  $9A                  ; $F832: 86 9A
        ldx.b  $A0                  ; $F834: A6 A0
        jsr    L_F6FE               ; $F836: 20 FE F6
; Conditional paths join or continue at L_F84B and L_F83F.
L_F839:
        ldx.b  $9A                  ; $F839: A6 9A
        cmp    #$51                 ; $F83B: C9 51
        bne    L_F84B               ; $F83D: D0 0C    ; 2/3 cycles
; Fall through to L_F84B.
L_F83F:
        lda    #$01                 ; $F83F: A9 01
        sta.b  NUSIZ0,x             ; $F841: 95 04
        lda    #$03                 ; $F843: A9 03
        sta.b  $E0                  ; $F845: 85 E0
        lda    #$10                 ; $F847: A9 10
        sta.b  $DF                  ; $F849: 85 DF
; Continue at L_F89E.
L_F84B:
        jmp    L_F89E               ; $F84B: 4C 9E F8
; Conditional paths join or continue at L_F89E and L_F852.
L_F84E:
        cmp    #$01                 ; $F84E: C9 01
        beq    L_F89E               ; $F850: F0 4C    ; 2/3 cycles
; Conditional paths join or continue at L_F871 and L_F856.
L_F852:
        cmp    #$02                 ; $F852: C9 02
        bne    L_F871               ; $F854: D0 1B    ; 2/3 cycles
; Continue at L_F89E.
L_F856:
        lda.b  $00,x                ; $F856: B5 00
        sta.b  $8A                  ; $F858: 85 8A
        sta.b  $E2                  ; $F85A: 85 E2
        lda.b  $01,x                ; $F85C: B5 01
        clc                         ; $F85E: 18
        adc    #$03                 ; $F85F: 69 03
        sta.b  $8B                  ; $F861: 85 8B
        sta.b  $E3                  ; $F863: 85 E3
        lda.b  $02,x                ; $F865: B5 02
        sec                         ; $F867: 38
        sbc    #$0A                 ; $F868: E9 0A
        sta.b  $8C                  ; $F86A: 85 8C
        sta.b  $E4                  ; $F86C: 85 E4
        jmp    L_F89E               ; $F86E: 4C 9E F8
; Conditional paths join or continue at L_F89E and L_F879.
L_F871:
        inc.b  $04,x                ; $F871: F6 04
        lda.b  $04,x                ; $F873: B5 04
        cmp    #$FC                 ; $F875: C9 FC
        bcc    L_F89E               ; $F877: 90 25    ; 2/3 cycles
; Call L_F6E9; continue at L_F87E.
L_F879:
        lda.b  $A0                  ; $F879: A5 A0
        jsr    L_F6E9               ; $F87B: 20 E9 F6
; Conditional paths join or continue at L_F89E and L_F880.
L_F87E:
        beq    L_F89E               ; $F87E: F0 1E    ; 2/3 cycles
; Conditional paths join or continue at L_F896 and L_F892.
L_F880:
        lda    #$02                 ; $F880: A9 02
        sta.b  NUSIZ0,x             ; $F882: 95 04
        lda    #$02                 ; $F884: A9 02
        sta.b  $E0                  ; $F886: 85 E0
        lda    #$10                 ; $F888: A9 10
        sta.b  $DF                  ; $F88A: 85 DF
        lda    #$9B                 ; $F88C: A9 9B
        cmp.b  $01,x                ; $F88E: D5 01
        beq    L_F896               ; $F890: F0 04    ; 2/3 cycles
; Conditional paths join or continue at L_F896 and L_F894.
L_F892:
        bcs    L_F896               ; $F892: B0 02    ; 2/3 cycles
; Fall through to L_F896.
L_F894:
        sta.b  VBLANK,x             ; $F894: 95 01
; Conditional paths join or continue at L_F89E and L_F89C.
L_F896:
        lda    #$17                 ; $F896: A9 17
        cmp.b  $02,x                ; $F898: D5 02
        bcc    L_F89E               ; $F89A: 90 02    ; 2/3 cycles
; Fall through to L_F89E.
L_F89C:
        sta.b  WSYNC,x              ; $F89C: 95 02    ; horizontal wait
; Return through the matching JSR continuation.
L_F89E:
        rts                         ; $F89E: 60

; Six actor-state values indexed by game selection and a console difficulty bit.
D_F89F:
        .byte $D0,$E8,$F0,$F6,$F0,$F6                          ; $F89F-$F8A4

; -----------------------------------------------------------------------------
; Update the carrying actor and its carried object using the list at $F927.
; JSR sites: $F342.
L_F8A5:
        inc.b  $CF                  ; $F8A5: E6 CF
        lda.b  $CF                  ; $F8A7: A5 CF
        cmp    #$08                 ; $F8A9: C9 08
        bne    L_F8B1               ; $F8AB: D0 04    ; 2/3 cycles
; Fall through to L_F8B1.
L_F8AD:
        lda    #$00                 ; $F8AD: A9 00
        sta.b  $CF                  ; $F8AF: 85 CF
; Conditional paths join or continue at L_F8C3 and L_F8B5.
L_F8B1:
        lda.b  $D1                  ; $F8B1: A5 D1
        beq    L_F8C3               ; $F8B3: F0 0E    ; 2/3 cycles
; Call L_F5FF; continue at L_F8C0.
L_F8B5:
        inc.b  $D1                  ; $F8B5: E6 D1
        lda.b  $CE                  ; $F8B7: A5 CE
        ldx    #$CB                 ; $F8B9: A2 CB
        ldy    #$03                 ; $F8BB: A0 03
        jsr    L_F5FF               ; $F8BD: 20 FF F5
; Continue at L_F908.
L_F8C0:
        jmp    L_F908               ; $F8C0: 4C 08 F9
; Call L_F715; continue at L_F8DA.
L_F8C3:
        lda    #$CB                 ; $F8C3: A9 CB
        sta.b  $D5                  ; $F8C5: 85 D5
        lda    #$03                 ; $F8C7: A9 03
        sta.b  $D4                  ; $F8C9: 85 D4
        lda    #<D_F927             ; $F8CB: A9 27
        sta.b  $D2                  ; $F8CD: 85 D2
        lda    #>D_F927             ; $F8CF: A9 F9
        sta.b  $D3                  ; $F8D1: 85 D3
        lda.b  $D0                  ; $F8D3: A5 D0
        sta.b  $D6                  ; $F8D5: 85 D6
        jsr    L_F715               ; $F8D7: 20 15 F7
; Conditional paths join or continue at L_F908 and L_F8E0.
L_F8DA:
        ldy.b  $E1                  ; $F8DA: A4 E1
        lda    ($D2),y              ; $F8DC: B1 D2
        beq    L_F908               ; $F8DE: F0 28    ; 2/3 cycles (+1 if taken across page)
; Conditional paths join or continue at L_F908 and L_F8EA.
L_F8E0:
        iny                         ; $F8E0: C8
        lda    ($D2),y              ; $F8E1: B1 D2
        tax                         ; $F8E3: AA
        lda.b  $00,x                ; $F8E4: B5 00
        cmp.b  $CB                  ; $F8E6: C5 CB
        bne    L_F908               ; $F8E8: D0 1E    ; 2/3 cycles (+1 if taken across page)
; Conditional paths join or continue at L_F908 and L_F8F6.
L_F8EA:
        lda.b  $01,x                ; $F8EA: B5 01
        sec                         ; $F8EC: 38
        sbc.b  $CC                  ; $F8ED: E5 CC
        clc                         ; $F8EF: 18
        adc    #$04                 ; $F8F0: 69 04
        and    #$F8                 ; $F8F2: 29 F8
        bne    L_F908               ; $F8F4: D0 12    ; 2/3 cycles (+1 if taken across page)
; Conditional paths join or continue at L_F908 and L_F902.
L_F8F6:
        lda.b  $02,x                ; $F8F6: B5 02
        sec                         ; $F8F8: 38
        sbc.b  $CD                  ; $F8F9: E5 CD
        clc                         ; $F8FB: 18
        adc    #$04                 ; $F8FC: 69 04
        and    #$F8                 ; $F8FE: 29 F8
        bne    L_F908               ; $F900: D0 06    ; 2/3 cycles
; Fall through to L_F908.
L_F902:
        stx.b  $D0                  ; $F902: 86 D0
        lda    #$10                 ; $F904: A9 10
        sta.b  $D1                  ; $F906: 85 D1
; Conditional paths join or continue at L_F926 and L_F922.
L_F908:
        ldx.b  $D0                  ; $F908: A6 D0
        lda.b  $CB                  ; $F90A: A5 CB
        sta.b  VSYNC,x              ; $F90C: 95 00
        lda.b  $CC                  ; $F90E: A5 CC
        clc                         ; $F910: 18
        adc    #$08                 ; $F911: 69 08
        sta.b  VBLANK,x             ; $F913: 95 01
        lda.b  $CD                  ; $F915: A5 CD
        sta.b  WSYNC,x              ; $F917: 95 02    ; horizontal wait
        lda.b  $D0                  ; $F919: A5 D0
        ldy.b  $9D                  ; $F91B: A4 9D
        cmp.w  D_FF44,y             ; $F91D: D9 44 FF
        bne    L_F926               ; $F920: D0 04    ; 2/3 cycles
; Fall through to L_F926.
L_F922:
        lda    #$A2                 ; $F922: A9 A2
        sta.b  $9D                  ; $F924: 85 9D
; Return through the matching JSR continuation.
L_F926:
        rts                         ; $F926: 60

; Ordered RAM-address pairs for the carrying actor search; zero terminates the list.
D_F927:
        .byte $CB,$B9,$CB,$B6,$CB,$BC,$CB,$BF,$CB,$C2,$CB,$C5  ; $F927-$F932
        .byte $CB,$A4,$CB,$A9,$CB,$AE,$CB,$B3,$00              ; $F933-$F93B

; -----------------------------------------------------------------------------
; Update the three gates using collision feedback and their required descriptor offsets.
; JSR sites: $F345.
L_F93C:
        ldy    #$02                 ; $F93C: A0 02
; Call L_F6FE; continue at L_F944.
L_F93E:
        ldx.w  D_F9A7,y             ; $F93E: BE A7 F9
        jsr    L_F6FE               ; $F941: 20 FE F6
; Conditional paths join or continue at L_F94F and L_F94B.
L_F944:
        sta.b  $97                  ; $F944: 85 97
        cmp.w  D_F9AA,y             ; $F946: D9 AA F9
        bne    L_F94F               ; $F949: D0 04    ; 2/3 cycles
; Fall through to L_F94F.
L_F94B:
        tya                         ; $F94B: 98
        tax                         ; $F94C: AA
        inc.b  $C8,x                ; $F94D: F6 C8
; Conditional paths join or continue at L_F988 and L_F957.
L_F94F:
        tya                         ; $F94F: 98
        tax                         ; $F950: AA
        lda.b  $C8,x                ; $F951: B5 C8
        cmp    #$1C                 ; $F953: C9 1C
        beq    L_F988               ; $F955: F0 31    ; 2/3 cycles
; Call L_F6E9; continue at L_F95D.
L_F957:
        lda.w  D_F9A7,y             ; $F957: B9 A7 F9
        jsr    L_F6E9               ; $F95A: 20 E9 F6
; Conditional paths join or continue at L_F968 and L_F95F.
L_F95D:
        beq    L_F968               ; $F95D: F0 09    ; 2/3 cycles
; Continue at L_F97F.
L_F95F:
        lda    #$01                 ; $F95F: A9 01
        sta.b  $C8,x                ; $F961: 95 C8
        ldx    #$8A                 ; $F963: A2 8A
        jmp    L_F97F               ; $F965: 4C 7F F9
; Conditional paths join or continue at L_F97C and L_F96E.
L_F968:
        lda.b  $97                  ; $F968: A5 97
        cmp    #$A2                 ; $F96A: C9 A2
        beq    L_F97C               ; $F96C: F0 0E    ; 2/3 cycles
; Call L_F2E4; continue at L_F975.
L_F96E:
        ldx.b  $97                  ; $F96E: A6 97
        sty.b  $9A                  ; $F970: 84 9A
        jsr    L_F2E4               ; $F972: 20 E4 F2
; Continue at L_F97F.
L_F975:
        ldy.b  $9A                  ; $F975: A4 9A
        ldx.b  $93                  ; $F977: A6 93
        jmp    L_F97F               ; $F979: 4C 7F F9
; Continue at L_F988.
L_F97C:
        jmp    L_F988               ; $F97C: 4C 88 F9
; Fall through to L_F988.
L_F97F:
        lda.w  D_F9AD,y             ; $F97F: B9 AD F9
        sta.b  VSYNC,x              ; $F982: 95 00
        lda    #$10                 ; $F984: A9 10
        sta.b  WSYNC,x              ; $F986: 95 02    ; horizontal wait
; Conditional paths join or continue at L_F9A0 and L_F990.
L_F988:
        tya                         ; $F988: 98
        tax                         ; $F989: AA
        lda.b  $C8,x                ; $F98A: B5 C8
        cmp    #$01                 ; $F98C: C9 01
        beq    L_F9A0               ; $F98E: F0 10    ; 2/3 cycles
; Conditional paths join or continue at L_F9A0 and L_F994.
L_F990:
        cmp    #$1C                 ; $F990: C9 1C
        beq    L_F9A0               ; $F992: F0 0C    ; 2/3 cycles
; Conditional paths join or continue at L_F9A0 and L_F99C.
L_F994:
        inc.b  $C8,x                ; $F994: F6 C8
        lda.b  $C8,x                ; $F996: B5 C8
        cmp    #$38                 ; $F998: C9 38
        bne    L_F9A0               ; $F99A: D0 04    ; 2/3 cycles
; Fall through to L_F9A0.
L_F99C:
        lda    #$01                 ; $F99C: A9 01
        sta.b  $C8,x                ; $F99E: 95 C8
; Conditional paths join or continue at L_F9A6 and L_F9A3.
L_F9A0:
        dey                         ; $F9A0: 88
        bmi    L_F9A6               ; $F9A1: 30 03    ; 2/3 cycles
; Continue at L_F93E.
L_F9A3:
        jmp    L_F93E               ; $F9A3: 4C 3E F9
; Return through the matching JSR continuation.
L_F9A6:
        rts                         ; $F9A6: 60

; Three object-descriptor offsets used by the gate update.
D_F9A7:
        .byte $09,$12,$1B                                      ; $F9A7-$F9A9

; Three descriptor offsets of the objects that open the corresponding gates.
D_F9AA:
        .byte $63,$6C,$75                                      ; $F9AA-$F9AC

; Three exterior room IDs associated with the gates.
D_F9AD:
        .byte $12,$1A,$1B                                      ; $F9AD-$F9AF

; Three interior room IDs associated with the gates.
D_F9B0:
        .byte $11,$0F,$10                                      ; $F9B0-$F9B2

; -----------------------------------------------------------------------------
; Temporarily adjust one actor coordinate, select a target, move it, and restore the adjustment.
; JSR sites: $F35C.
L_F9B3:
        lda.b  $B5                  ; $F9B3: A5 B5
        sec                         ; $F9B5: 38
        sbc    #$08                 ; $F9B6: E9 08
        sta.b  $B5                  ; $F9B8: 85 B5
        lda    #$00                 ; $F9BA: A9 00
        sta.b  $D6                  ; $F9BC: 85 D6
        lda    #<D_F9DA             ; $F9BE: A9 DA
        sta.b  $D2                  ; $F9C0: 85 D2
        lda    #>D_F9DA             ; $F9C2: A9 F9
        sta.b  $D3                  ; $F9C4: 85 D3
        jsr    L_F728               ; $F9C6: 20 28 F7
; Conditional paths join or continue at L_F9D2 and L_F9CD.
L_F9C9:
        lda.b  $9B                  ; $F9C9: A5 9B
        beq    L_F9D2               ; $F9CB: F0 05    ; 2/3 cycles
; Call L_F5FF; continue at L_F9D2.
L_F9CD:
        ldy    #$01                 ; $F9CD: A0 01
        jsr    L_F5FF               ; $F9CF: 20 FF F5
; Return through the matching JSR continuation.
L_F9D2:
        lda.b  $B5                  ; $F9D2: A5 B5
        clc                         ; $F9D4: 18
        adc    #$08                 ; $F9D5: 69 08
        sta.b  $B5                  ; $F9D7: 85 B5
        rts                         ; $F9D9: 60

; Ordered RAM-address pairs for the final actor search; zero terminates the list.
D_F9DA:
        .byte $BF,$B3,$C2,$B3,$C5,$B3,$B6,$B3,$BC,$B3,$B9,$B3  ; $F9DA-$F9E5
        .byte $00                                              ; $F9E6-$F9E6

; -----------------------------------------------------------------------------
; Prepare the special displayed-object coordinates for rooms whose color descriptor is $08.
; JSR sites: $F33C.
L_F9E7:
        lda.b  $8A                  ; $F9E7: A5 8A
        jsr    L_F271               ; $F9E9: 20 71 F2
; Conditional paths join or continue at L_F9FB and L_F9F4.
L_F9EC:
        ldy    #$02                 ; $F9EC: A0 02
        lda    ($93),y              ; $F9EE: B1 93
        cmp    #$08                 ; $F9F0: C9 08
        beq    L_F9FB               ; $F9F2: F0 07    ; 2/3 cycles
; Continue at L_FA22.
L_F9F4:
        lda    #$00                 ; $F9F4: A9 00
        sta.b  $DB                  ; $F9F6: 85 DB
        jmp    L_FA22               ; $F9F8: 4C 22 FA
; Conditional paths join or continue at L_FA1A and L_FA13.
L_F9FB:
        lda.b  $8A                  ; $F9FB: A5 8A
        sta.b  $D9                  ; $F9FD: 85 D9
        lda.b  $8B                  ; $F9FF: A5 8B
        sec                         ; $FA01: 38
        sbc    #$0E                 ; $FA02: E9 0E
        sta.b  $DA                  ; $FA04: 85 DA
        lda.b  $8C                  ; $FA06: A5 8C
        clc                         ; $FA08: 18
        adc    #$0E                 ; $FA09: 69 0E
        sta.b  $DB                  ; $FA0B: 85 DB
        lda.b  $DA                  ; $FA0D: A5 DA
        cmp    #$F0                 ; $FA0F: C9 F0
        bcc    L_FA1A               ; $FA11: 90 07    ; 2/3 cycles
; Continue at L_FA22.
L_FA13:
        lda    #$01                 ; $FA13: A9 01
        sta.b  $DA                  ; $FA15: 85 DA
        jmp    L_FA22               ; $FA17: 4C 22 FA
; Conditional paths join or continue at L_FA22 and L_FA1E.
L_FA1A:
        cmp    #$82                 ; $FA1A: C9 82
        bcc    L_FA22               ; $FA1C: 90 04    ; 2/3 cycles
; Fall through to L_FA22.
L_FA1E:
        lda    #$81                 ; $FA1E: A9 81
        sta.b  $DA                  ; $FA20: 85 DA
; Return through the matching JSR continuation.
L_FA22:
        rts                         ; $FA22: 60

; -----------------------------------------------------------------------------
; Advance the effect countdown; dispatch six audio/effect patterns or silence the volume registers.
; JSR sites: $F309.
L_FA23:
        lda.b  $DF                  ; $FA23: A5 DF
        bne    L_FA2C               ; $FA25: D0 05    ; 2/3 cycles
; Return through the matching JSR continuation.
L_FA27:
        sta.b  AUDV0                ; $FA27: 85 19
        sta.b  AUDV1                ; $FA29: 85 1A
        rts                         ; $FA2B: 60
; Conditional paths join or continue at L_FA47 and L_FA32.
L_FA2C:
        dec.b  $DF                  ; $FA2C: C6 DF
        lda.b  $E0                  ; $FA2E: A5 E0
        beq    L_FA47               ; $FA30: F0 15    ; 2/3 cycles
; Conditional paths join or continue at L_FA55 and L_FA36.
L_FA32:
        cmp    #$01                 ; $FA32: C9 01
        beq    L_FA55               ; $FA34: F0 1F    ; 2/3 cycles
; Conditional paths join or continue at L_FA6C and L_FA3A.
L_FA36:
        cmp    #$02                 ; $FA36: C9 02
        beq    L_FA6C               ; $FA38: F0 32    ; 2/3 cycles
; Conditional paths join or continue at L_FA7F and L_FA3E.
L_FA3A:
        cmp    #$03                 ; $FA3A: C9 03
        beq    L_FA7F               ; $FA3C: F0 41    ; 2/3 cycles
; Conditional paths join or continue at L_FA8C and L_FA42.
L_FA3E:
        cmp    #$04                 ; $FA3E: C9 04
        beq    L_FA8C               ; $FA40: F0 4A    ; 2/3 cycles
; Conditional paths join or continue at L_FA9B and L_FA46.
L_FA42:
        cmp    #$05                 ; $FA42: C9 05
        beq    L_FA9B               ; $FA44: F0 55    ; 2/3 cycles
; Return through the matching JSR continuation.
L_FA46:
        rts                         ; $FA46: 60
; Return through the matching JSR continuation.
L_FA47:
        lda.b  $DF                  ; $FA47: A5 DF
        sta.b  COLUPF               ; $FA49: 85 08
        sta.b  AUDC0                ; $FA4B: 85 15
        lsr                         ; $FA4D: 4A
        sta.b  AUDV0                ; $FA4E: 85 19
        lsr                         ; $FA50: 4A
        lsr                         ; $FA51: 4A
        sta.b  AUDF0                ; $FA52: 85 17
        rts                         ; $FA54: 60
; Conditional paths join or continue at L_FA5E and L_FA5C.
L_FA55:
        lda.b  $DF                  ; $FA55: A5 DF
        lsr                         ; $FA57: 4A
        lda    #$03                 ; $FA58: A9 03
        bcs    L_FA5E               ; $FA5A: B0 02    ; 2/3 cycles
; Fall through to L_FA5E.
L_FA5C:
        lda    #$08                 ; $FA5C: A9 08
; Return through the matching JSR continuation.
L_FA5E:
        sta.b  AUDC0                ; $FA5E: 85 15
        lda.b  $DF                  ; $FA60: A5 DF
        sta.b  AUDV0                ; $FA62: 85 19
        lsr                         ; $FA64: 4A
        lsr                         ; $FA65: 4A
        clc                         ; $FA66: 18
        adc    #$1C                 ; $FA67: 69 1C
        sta.b  AUDF0                ; $FA69: 85 17
        rts                         ; $FA6B: 60
; Return through the matching JSR continuation.
L_FA6C:
        lda    #$06                 ; $FA6C: A9 06
        sta.b  AUDC0                ; $FA6E: 85 15
        lda.b  $DF                  ; $FA70: A5 DF
        eor    #$0F                 ; $FA72: 49 0F
        sta.b  AUDF0                ; $FA74: 85 17
        lda.b  $DF                  ; $FA76: A5 DF
        lsr                         ; $FA78: 4A
        clc                         ; $FA79: 18
        adc    #$08                 ; $FA7A: 69 08
        sta.b  AUDV0                ; $FA7C: 85 19
        rts                         ; $FA7E: 60
; Return through the matching JSR continuation.
L_FA7F:
        lda    #$04                 ; $FA7F: A9 04
        sta.b  AUDC0                ; $FA81: 85 15
        lda.b  $DF                  ; $FA83: A5 DF
        sta.b  AUDV0                ; $FA85: 85 19
        eor    #$1F                 ; $FA87: 49 1F
        sta.b  AUDF0                ; $FA89: 85 17
        rts                         ; $FA8B: 60
; Fall through to L_FA90.
L_FA8C:
        lda.b  $DF                  ; $FA8C: A5 DF
        eor    #$03                 ; $FA8E: 49 03
; Shared audio tail, also entered by the effect at $FA9B.
L_FA90:
        sta.b  AUDF0                ; $FA90: 85 17
        lda    #$05                 ; $FA92: A9 05
        sta.b  AUDV0                ; $FA94: 85 19
        lda    #$06                 ; $FA96: A9 06
        sta.b  AUDC0                ; $FA98: 85 15
        rts                         ; $FA9A: 60
; Continue at L_FA90.
L_FA9B:
        lda.b  $DF                  ; $FA9B: A5 DF
        jmp    L_FA90               ; $FA9D: 4C 90 FA

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FAA0:
        .byte $F0,$FF,$FF,$00,$00,$00,$00,$00,$00,$00,$00,$00  ; $FAA0-$FAAB
        .byte $00,$00,$00,$00,$00,$00                          ; $FAAC-$FAB1

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FAB2:
        .byte $F0,$FF,$0F,$00,$00,$00,$00,$00,$00,$00,$00,$00  ; $FAB2-$FABD
        .byte $00,$00,$00,$00,$00,$00,$F0,$FF,$FF              ; $FABE-$FAC6

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FAC7:
        .byte $F0,$FF,$0F,$00,$00,$00,$00,$00,$00,$00,$00,$00  ; $FAC7-$FAD2
        .byte $00,$00,$00,$00,$00,$00,$F0,$FF,$0F              ; $FAD3-$FADB

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FADC:
        .byte $F0,$FF,$FF,$30,$00,$00,$30,$00,$00,$30,$00,$00  ; $FADC-$FAE7
        .byte $30,$00,$00,$30,$00,$00,$F0,$FF,$0F              ; $FAE8-$FAF0

; Graphics selection triples (inclusive state ceiling, little-endian bitmap pointer).
D_FAF1:
        .byte $04,$24,$FB,$08,$22,$FB,$0C,$20,$FB,$10,$1E,$FB  ; $FAF1-$FAFC
        .byte $14,$1C,$FB,$18,$1A,$FB,$1C,$18,$FB,$20,$1A,$FB  ; $FAFD-$FB08
        .byte $24,$1C,$FB,$28,$1E,$FB,$2C,$20,$FB,$30,$22,$FB  ; $FB09-$FB14
        .byte $FF,$24,$FB                                      ; $FB15-$FB17

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FB18:
        .byte $FE,$AA                                          ; $FB18-$FB19

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FB1A:
        .byte $FE,$AA                                          ; $FB1A-$FB1B

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FB1C:
        .byte $FE,$AA                                          ; $FB1C-$FB1D

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FB1E:
        .byte $FE,$AA                                          ; $FB1E-$FB1F

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FB20:
        .byte $FE,$AA                                          ; $FB20-$FB21

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FB22:
        .byte $FE,$AA                                          ; $FB22-$FB23

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FB24:
        .byte $FE,$AA,$FE,$AA,$00                              ; $FB24-$FB28

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FB29:
        .byte $F0,$FF,$0F,$30,$00,$00,$30,$00,$00,$30,$00,$00  ; $FB29-$FB34
        .byte $30,$00,$00,$30,$00,$00,$F0,$FF,$0F              ; $FB35-$FB3D

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FB3E:
        .byte $F0,$FF,$0F,$00,$0C,$0C,$F0,$0C,$3C,$F0,$0C,$00  ; $FB3E-$FB49
        .byte $F0,$FF,$3F,$00,$30,$30,$F0,$33,$3F              ; $FB4A-$FB52

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FB53:
        .byte $F0,$FF,$FF,$00,$00,$00,$F0,$FC,$FF,$F0,$00,$C0  ; $FB53-$FB5E
        .byte $F0,$3F,$CF,$00,$30,$CC,$F0,$F3,$CC              ; $FB5F-$FB67

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FB68:
        .byte $F0,$F3,$0C,$00,$30,$0C,$F0,$3F,$0F,$F0,$00,$00  ; $FB68-$FB73
        .byte $F0,$F0,$00,$00,$30,$00,$F0,$FF,$FF              ; $FB74-$FB7C

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FB7D:
        .byte $F0,$33,$3F,$00,$30,$3C,$F0,$FF,$3C,$00,$03,$3C  ; $FB7D-$FB88
        .byte $F0,$33,$3C,$00,$33,$0C,$F0,$F3,$0C              ; $FB89-$FB91

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FB92:
        .byte $F0,$F3,$CC,$00,$33,$0C,$F0,$33,$FC,$00,$33,$00  ; $FB92-$FB9D
        .byte $F0,$F3,$FF,$00,$00,$00,$F0,$FF,$0F              ; $FB9E-$FBA6

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FBA7:
        .byte $F0,$FF,$CC,$00,$00,$CC,$F0,$03,$CF,$00,$03,$00  ; $FBA7-$FBB2
        .byte $F0,$F3,$FC,$00,$33,$0C                          ; $FBB3-$FBB8

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FBB9:
        .byte $F0,$33,$CC,$00,$30,$CC,$00,$3F,$CF,$00,$00,$C0  ; $FBB9-$FBC4
        .byte $00,$3F,$C3,$00,$30,$C0,$F0,$FF,$FF              ; $FBC5-$FBCD

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FBCE:
        .byte $F0,$FF,$0F,$00,$30,$00,$F0,$30,$FF,$00,$30,$C0  ; $FBCE-$FBD9
        .byte $F0,$F3,$C0,$00,$03,$C0,$F0,$FF,$CC              ; $FBDA-$FBE2

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FBE3:
        .byte $F0,$FE,$15,$30,$03,$1F,$30,$03,$FF,$30,$00,$FF  ; $FBE3-$FBEE
        .byte $30,$00,$3F,$30,$00,$00,$F0,$FF,$0F              ; $FBEF-$FBF7

; Three fixed room/x/y records for the gate objects ($FBF8, $FBFB, $FBFE).
D_FBF8:
        .byte $11,$4D,$31                                      ; $FBF8-$FBFA

; Fixed room/x/y record referenced by descriptor $FF56.
D_FBFB:
        .byte $0F,$4D,$31                                      ; $FBFB-$FBFD

; Fixed room/x/y record referenced by descriptor $FF5F.
D_FBFE:
        .byte $10,$4D,$31                                      ; $FBFE-$FC00

; Fixed object-state byte referenced by an object descriptor.
D_FC01:
        .byte $00                                              ; $FC01-$FC01

; Graphics selection triples (inclusive state ceiling, little-endian bitmap pointer).
D_FC02:
        .byte $FF,$05,$FC                                      ; $FC02-$FC04

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FC05:
        .byte $FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF  ; $FC05-$FC10
        .byte $FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF  ; $FC11-$FC1C
        .byte $FF,$FF,$FF,$FF,$FF,$FF,$FF,$FF,$00              ; $FC1D-$FC25

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FC26:
        .byte $F0,$FF,$FF,$00,$00,$00,$F0,$FF,$0F,$00,$00,$0C  ; $FC26-$FC31
        .byte $F0,$FF,$0C,$F0,$03,$CC                          ; $FC32-$FC37

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FC38:
        .byte $F0,$33,$CF,$F0,$30,$00,$F0,$33,$FF,$00,$33,$00  ; $FC38-$FC43
        .byte $F0,$FF,$00,$00,$00,$00,$F0,$FF,$0F              ; $FC44-$FC4C

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FC4D:
        .byte $F0,$FF,$FF,$00,$00,$C0,$F0,$FF,$CF,$00,$00,$CC  ; $FC4D-$FC58
        .byte $F0,$33,$FF,$F0,$33,$00                          ; $FC59-$FC5E

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FC5F:
        .byte $F0,$3F,$0C,$F0,$00,$0C,$F0,$FF,$0F,$00,$30,$00  ; $FC5F-$FC6A
        .byte $F0,$30,$00,$00,$30,$00,$F0,$FF,$0F              ; $FC6B-$FC73

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FC74:
        .byte $F0,$FF,$0F,$30,$00,$00,$30,$00,$00,$30,$00,$00  ; $FC74-$FC7F
        .byte $30,$00,$00,$30,$00,$00,$F0,$FF,$FF              ; $FC80-$FC88

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FC89:
        .byte $F0,$F0,$FF,$00,$00,$03,$F0,$FF,$03,$00,$00,$00  ; $FC89-$FC94
        .byte $30,$3F,$FF,$00,$30,$00                          ; $FC95-$FC9A

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FC9B:
        .byte $F0,$F0,$FF,$30,$00,$00,$30,$3F,$FF,$00,$30,$00  ; $FC9B-$FCA6
        .byte $F0,$F0,$FF,$30,$00,$03,$F0,$F0,$FF              ; $FCA7-$FCAF

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FCB0:
        .byte $F0,$FF,$FF,$00,$00,$C0,$F0,$FF,$CF,$00,$00,$0C  ; $FCB0-$FCBB
        .byte $F0,$0F,$FF,$00,$0F,$C0                          ; $FCBC-$FCC1

; Playfield byte stream referenced by room records: repeating PF0/PF1/PF2 triples.
D_FCC2:
        .byte $30,$CF,$CC,$00,$C0,$CC,$F0,$FF,$0F,$00,$00,$00  ; $FCC2-$FCCD
        .byte $F0,$FF,$0F,$00,$00,$00,$F0,$FF,$0F              ; $FCCE-$FCD6

; Fixed object-state byte referenced by an object descriptor.
D_FCD7:
        .byte $00                                              ; $FCD7-$FCD7

; Graphics selection triples (inclusive state ceiling, little-endian bitmap pointer).
D_FCD8:
        .byte $FF,$DB,$FC                                      ; $FCD8-$FCDA

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FCDB:
        .byte $C3,$C3,$C3,$C3,$42,$42,$42,$42,$42,$42,$42,$42  ; $FCDB-$FCE6
        .byte $42,$42,$42,$42,$42,$42,$42,$42,$C3,$C3,$C3,$C3  ; $FCE7-$FCF2
        .byte $00                                              ; $FCF3-$FCF3

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FCF4:
        .byte $04,$0C,$04,$04,$04,$04,$0E,$00                  ; $FCF4-$FCFB

; Fixed object-state byte referenced by an object descriptor.
D_FCFC:
        .byte $00                                              ; $FCFC-$FCFC

; Graphics selection triples (inclusive state ceiling, little-endian bitmap pointer).
D_FCFD:
        .byte $FF,$00,$FD                                      ; $FCFD-$FCFF

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FD00:
        .byte $07,$FD,$A7,$00                                  ; $FD00-$FD03

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FD04:
        .byte $0E,$11,$01,$02,$04,$08,$1F,$00                  ; $FD04-$FD0B

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FD0C:
        .byte $0E,$11,$01,$06,$01,$11,$0E,$00                  ; $FD0C-$FD13

; Graphics selection triples (inclusive state ceiling, little-endian bitmap pointer).
D_FD14:
        .byte $03,$1A,$FD,$FF,$22,$FD                          ; $FD14-$FD19

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FD1A:
        .byte $81,$81,$C3,$C3,$FF,$5A,$66,$00                  ; $FD1A-$FD21

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FD22:
        .byte $01,$80,$01,$80,$3C,$5A,$66,$C3,$81,$81,$81,$00  ; $FD22-$FD2D

; Graphics selection triples (inclusive state ceiling, little-endian bitmap pointer).
D_FD2E:
        .byte $00,$3A,$FD,$01,$66,$FD,$02,$3A,$FD,$FF,$4F,$FD  ; $FD2E-$FD39

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FD3A:
        .byte $06,$0F,$F3,$FE,$0E,$04,$04,$1E,$3F,$7F,$E3,$C3  ; $FD3A-$FD45
        .byte $C3,$C7,$FF,$3C,$08,$8F,$E1,$3F,$00              ; $FD46-$FD4E

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FD4F:
        .byte $80,$40,$26,$1F,$0B,$0E,$1E,$24,$44,$8E,$1E,$3F  ; $FD4F-$FD5A
        .byte $7F,$7F,$7F,$7F,$3E,$1C,$08,$F8,$80,$E0,$00      ; $FD5B-$FD65

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FD66:
        .byte $0C,$0C,$0C,$0E,$1B,$7F,$CE,$80,$FC,$FE,$FE,$7E  ; $FD66-$FD71
        .byte $78,$20,$6E,$42,$7E,$00                          ; $FD72-$FD77

; Fixed object-state byte referenced by an object descriptor.
D_FD78:
        .byte $00                                              ; $FD78-$FD78

; Graphics selection triples (inclusive state ceiling, little-endian bitmap pointer).
D_FD79:
        .byte $FF,$7C,$FD                                      ; $FD79-$FD7B

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FD7C:
        .byte $20,$40,$FF,$40,$20,$00                          ; $FD7C-$FD81

; Fixed object-state byte referenced by an object descriptor.
D_FD82:
        .byte $00                                              ; $FD82-$FD82

; Graphics selection triples (inclusive state ceiling, little-endian bitmap pointer).
D_FD83:
        .byte $FF,$86,$FD                                      ; $FD83-$FD85

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FD86:
        .byte $80,$00                                          ; $FD86-$FD87

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FD88:
        .byte $F0,$80,$80,$80,$F4,$04,$87,$E5,$87,$80,$05,$E5  ; $FD88-$FD93
        .byte $A7,$E1,$87,$E0,$01,$E0,$A0,$F0,$01,$40,$E0,$40  ; $FD94-$FD9F
        .byte $40,$40,$01,$E0,$A0,$E0,$80,$E0,$01,$20,$20,$E0  ; $FDA0-$FDAB
        .byte $A0,$E0,$01,$01,$01,$88,$A8,$A8,$A8,$F8,$01,$E0  ; $FDAC-$FDB7
        .byte $A0,$F0,$01,$80,$E0,$8F,$89,$0F,$8A,$E9,$80,$8E  ; $FDB8-$FDC3
        .byte $0A,$EE,$A0,$E8,$88,$EE,$0A,$8E,$E0,$A4,$A4,$04  ; $FDC4-$FDCF
        .byte $80,$08,$0E,$0A,$0A,$80,$0E,$0A,$0E,$08,$0E,$80  ; $FDD0-$FDDB
        .byte $04,$0E,$04,$04,$04,$80,$04,$0E,$04,$04,$04,$00  ; $FDDC-$FDE7

; Fixed room/x/y record referenced by an object descriptor.
D_FDE8:
        .byte $1E,$50,$69                                      ; $FDE8-$FDEA

; Fixed object-state byte referenced by an object descriptor.
D_FDEB:
        .byte $00                                              ; $FDEB-$FDEB

; Graphics selection triples (inclusive state ceiling, little-endian bitmap pointer).
D_FDEC:
        .byte $FF,$88,$FD                                      ; $FDEC-$FDEE

; Fixed object-state byte referenced by an object descriptor.
D_FDEF:
        .byte $00                                              ; $FDEF-$FDEF

; Graphics selection triples (inclusive state ceiling, little-endian bitmap pointer).
D_FDF0:
        .byte $FF,$F3,$FD                                      ; $FDF0-$FDF2

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FDF3:
        .byte $81,$81,$C3,$7E,$7E,$3C,$18,$18,$7E,$00          ; $FDF3-$FDFC

; Fixed object-state byte referenced by an object descriptor.
D_FDFD:
        .byte $00                                              ; $FDFD-$FDFD

; Graphics selection triples (inclusive state ceiling, little-endian bitmap pointer).
D_FDFE:
        .byte $FF,$01,$FE                                      ; $FDFE-$FE00

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FE01:
        .byte $00                                              ; $FE01-$FE01

; Fixed room/x/y record referenced by an object descriptor.
D_FE02:
        .byte $00,$50,$40                                      ; $FE02-$FE04

; Graphics selection triples (inclusive state ceiling, little-endian bitmap pointer).
D_FE05:
        .byte $01,$F4,$FC,$03,$04,$FD,$FF,$0C,$FD              ; $FE05-$FE0D

; Fixed object-state byte referenced by an object descriptor.
D_FE0E:
        .byte $00                                              ; $FE0E-$FE0E

; Graphics selection triples (inclusive state ceiling, little-endian bitmap pointer).
D_FE0F:
        .byte $FF,$12,$FE                                      ; $FE0F-$FE11

; Sprite byte stream; the kernel holds each byte for two lines. Zero ends advancement.
D_FE12:
        .byte $3C,$7E,$E7,$C3,$C3,$C3,$C3,$C3,$00              ; $FE12-$FE1A

; Room record 0: playfield pointer, two colors, CTRLPF, four exits.
D_FE1B:
        .byte $DC,$FA,$66,$0A,$21,$00,$00,$00,$00              ; $FE1B-$FE23

; Room record 1: playfield pointer, two colors, CTRLPF, four exits.
D_FE24:
        .byte $B2,$FA,$D8,$0A,$A1,$08,$02,$80,$03              ; $FE24-$FE2C

; Room record 2: playfield pointer, two colors, CTRLPF, four exits.
D_FE2D:
        .byte $B2,$FA,$C8,$0A,$21,$11,$03,$83,$01              ; $FE2D-$FE35

; Room record 3: playfield pointer, two colors, CTRLPF, four exits.
D_FE36:
        .byte $A0,$FA,$E8,$0A,$61,$06,$01,$86,$02              ; $FE36-$FE3E

; Room record 4: playfield pointer, two colors, CTRLPF, four exits.
D_FE3F:
        .byte $3E,$FB,$86,$0A,$21,$10,$05,$07,$06              ; $FE3F-$FE47

; Room record 5: playfield pointer, two colors, CTRLPF, four exits.
D_FE48:
        .byte $53,$FB,$86,$0A,$21,$1D,$06,$08,$04              ; $FE48-$FE50

; Room record 6: playfield pointer, two colors, CTRLPF, four exits.
D_FE51:
        .byte $68,$FB,$86,$0A,$21,$07,$04,$03,$05              ; $FE51-$FE59

; Room record 7: playfield pointer, two colors, CTRLPF, four exits.
D_FE5A:
        .byte $7D,$FB,$86,$0A,$21,$04,$08,$06,$08              ; $FE5A-$FE62

; Room record 8: playfield pointer, two colors, CTRLPF, four exits.
D_FE63:
        .byte $92,$FB,$86,$0A,$21,$05,$07,$01,$07              ; $FE63-$FE6B

; Room record 9: playfield pointer, two colors, CTRLPF, four exits.
D_FE6C:
        .byte $A7,$FB,$08,$08,$25,$0A,$0A,$0B,$0A              ; $FE6C-$FE74

; Room record 10: playfield pointer, two colors, CTRLPF, four exits.
D_FE75:
        .byte $CE,$FB,$08,$08,$25,$03,$09,$09,$09              ; $FE75-$FE7D

; Room record 11: playfield pointer, two colors, CTRLPF, four exits.
D_FE7E:
        .byte $B9,$FB,$08,$08,$25,$09,$0C,$1C,$0D              ; $FE7E-$FE86

; Room record 12: playfield pointer, two colors, CTRLPF, four exits.
D_FE87:
        .byte $C7,$FA,$98,$0A,$61,$1C,$0D,$1D,$0B              ; $FE87-$FE8F

; Room record 13: playfield pointer, two colors, CTRLPF, four exits.
D_FE90:
        .byte $C7,$FA,$B8,$0A,$A1,$0F,$0B,$0E,$0C              ; $FE90-$FE98

; Room record 14: playfield pointer, two colors, CTRLPF, four exits.
D_FE99:
        .byte $74,$FC,$A8,$0A,$21,$0D,$10,$0F,$10              ; $FE99-$FEA1

; Room record 15: playfield pointer, two colors, CTRLPF, four exits.
D_FEA2:
        .byte $E3,$FB,$0C,$0C,$21,$0E,$0F,$0D,$0F              ; $FEA2-$FEAA

; Room record 16: playfield pointer, two colors, CTRLPF, four exits.
D_FEAB:
        .byte $E3,$FB,$00,$02,$21,$01,$1C,$04,$1C              ; $FEAB-$FEB3

; Room record 17: playfield pointer, two colors, CTRLPF, four exits.
D_FEB4:
        .byte $E3,$FB,$1A,$0A,$21,$06,$03,$02,$01              ; $FEB4-$FEBC

; Room record 18: playfield pointer, two colors, CTRLPF, four exits.
D_FEBD:
        .byte $DC,$FA,$1A,$0A,$21,$12,$12,$12,$12              ; $FEBD-$FEC5

; Room record 19: playfield pointer, two colors, CTRLPF, four exits.
D_FEC6:
        .byte $89,$FC,$08,$08,$25,$15,$14,$15,$16              ; $FEC6-$FECE

; Room record 20: playfield pointer, two colors, CTRLPF, four exits.
D_FECF:
        .byte $B0,$FC,$08,$08,$24,$16,$15,$16,$13              ; $FECF-$FED7

; Room record 21: playfield pointer, two colors, CTRLPF, four exits.
D_FED8:
        .byte $9B,$FC,$08,$08,$24,$13,$16,$13,$14              ; $FED8-$FEE0

; Room record 22: playfield pointer, two colors, CTRLPF, four exits.
D_FEE1:
        .byte $C2,$FC,$08,$08,$25,$14,$13,$1B,$15              ; $FEE1-$FEE9

; Room record 23: playfield pointer, two colors, CTRLPF, four exits.
D_FEEA:
        .byte $26,$FC,$36,$0A,$21,$19,$18,$19,$18              ; $FEEA-$FEF2

; Room record 24: playfield pointer, two colors, CTRLPF, four exits.
D_FEF3:
        .byte $4D,$FC,$36,$0A,$21,$1A,$17,$1A,$17              ; $FEF3-$FEFB

; Room record 25: playfield pointer, two colors, CTRLPF, four exits.
D_FEFC:
        .byte $38,$FC,$36,$0A,$21,$17,$1A,$17,$1A              ; $FEFC-$FF04

; Room record 26: playfield pointer, two colors, CTRLPF, four exits.
D_FF05:
        .byte $5F,$FC,$36,$0A,$21,$18,$19,$18,$19              ; $FF05-$FF0D

; Room record 27: playfield pointer, two colors, CTRLPF, four exits.
D_FF0E:
        .byte $29,$FB,$36,$0A,$21,$89,$89,$89,$89              ; $FF0E-$FF16

; Room record 28: playfield pointer, two colors, CTRLPF, four exits.
D_FF17:
        .byte $DC,$FA,$66,$0A,$21,$1D,$07,$8C,$08              ; $FF17-$FF1F

; Room record 29: playfield pointer, two colors, CTRLPF, four exits.
D_FF20:
        .byte $74,$FC,$36,$0A,$21,$8F,$01,$10,$03              ; $FF20-$FF28

; Room record 30: playfield pointer, two colors, CTRLPF, four exits.
D_FF29:
        .byte $B2,$FA,$66,$0A,$21,$06,$01,$06,$03              ; $FF29-$FF31

; 18 room-transition values: six groups of three, indexed by tagged exit and game selection.
D_FF32:
        .byte $10,$0F,$0F,$05,$11,$11,$1D,$0A,$0A,$1C,$16,$16  ; $FF32-$FF3D
        .byte $1B,$0C,$0C,$03,$0C,$0C                          ; $FF3E-$FF43

; Object descriptor offset $00: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FF44:
        .byte $D9                                              ; $FF44-$FF44

; Interior table address read directly by code.
D_FF45:
        .byte $00                                              ; $FF45-$FF45

; Interior table address read directly by code.
D_FF46:
        .byte $01                                              ; $FF46-$FF46

; Interior table address read directly by code.
D_FF47:
        .byte $FC                                              ; $FF47-$FF47

; Interior table address read directly by code.
D_FF48:
        .byte $02                                              ; $FF48-$FF48

; Interior table address read directly by code.
D_FF49:
        .byte $FC                                              ; $FF49-$FF49

; Interior table address read directly by code.
D_FF4A:
        .byte $28                                              ; $FF4A-$FF4A

; Interior table address read directly by code.
D_FF4B:
        .byte $0C                                              ; $FF4B-$FF4B

; Interior table address read directly by code.
D_FF4C:
        .byte $07                                              ; $FF4C-$FF4C

; Object descriptor offset $09: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FF4D:
        .byte $F8,$FB,$C8,$00,$F1,$FA,$00,$00,$00              ; $FF4D-$FF55

; Object descriptor offset $12: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FF56:
        .byte $FB,$FB,$C9,$00,$F1,$FA,$00,$00,$00              ; $FF56-$FF5E

; Object descriptor offset $1B: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FF5F:
        .byte $FE,$FB,$CA,$00,$F1,$FA,$00,$00,$00              ; $FF5F-$FF67

; Object descriptor offset $24: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FF68:
        .byte $E8,$FD,$EB,$FD,$EC,$FD,$CB,$00,$00              ; $FF68-$FF70

; Object descriptor offset $2D: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FF71:
        .byte $02,$FE,$DD,$00,$05,$FE,$C8,$00,$00              ; $FF71-$FF79

; Object descriptor offset $36: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FF7A:
        .byte $A4,$00,$A8,$00,$2E,$FD,$36,$0E,$00              ; $FF7A-$FF82

; Object descriptor offset $3F: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FF83:
        .byte $A9,$00,$AD,$00,$2E,$FD,$1A,$06,$00              ; $FF83-$FF8B

; Object descriptor offset $48: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FF8C:
        .byte $AE,$00,$B2,$00,$2E,$FD,$C8,$00,$00              ; $FF8C-$FF94

; Object descriptor offset $51: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FF95:
        .byte $B6,$00,$78,$FD,$79,$FD,$1A,$06,$00              ; $FF95-$FF9D

; Object descriptor offset $5A: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FF9E:
        .byte $BC,$00,$D7,$FC,$D8,$FC,$66,$02,$07              ; $FF9E-$FFA6

; Object descriptor offset $63: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FFA7:
        .byte $BF,$00,$FC,$FC,$FD,$FC,$1A,$06,$00              ; $FFA7-$FFAF

; Object descriptor offset $6C: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FFB0:
        .byte $C2,$00,$FC,$FC,$FD,$FC,$0E,$0E,$00              ; $FFB0-$FFB8

; Object descriptor offset $75: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FFB9:
        .byte $C5,$00,$FC,$FC,$FD,$FC,$00,$00,$00              ; $FFB9-$FFC1

; Object descriptor offset $7E: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FFC2:
        .byte $CB,$00,$CF,$00,$14,$FD,$00,$00,$00              ; $FFC2-$FFCA

; Object descriptor offset $87: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FFCB:
        .byte $A1,$00,$82,$FD,$83,$FD,$08,$08,$00              ; $FFCB-$FFD3

; Object descriptor offset $90: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FFD4:
        .byte $B9,$00,$EF,$FD,$F0,$FD,$CB,$06,$00              ; $FFD4-$FFDC

; Object descriptor offset $99: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FFDD:
        .byte $B3,$00,$0E,$FE,$0F,$FE,$00,$06,$00              ; $FFDD-$FFE5

; Object descriptor offset $A2: position ptr, state ptr, graphics-selection ptr, two colors, NUSIZ.
D_FFE6:
        .byte $BC,$00,$FD,$FD,$FE,$FD,$00,$00,$00              ; $FFE6-$FFEE

; Eleven trailing zero bytes with no identified ROM reader; preserved as padding.
D_FFEF:
        .byte $00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00      ; $FFEF-$FFF9

; NMI vector (not driven on the 6507); RESET and IRQ/BRK words follow.
D_FFFA:
        .word L_F000                 ; $FFFA: 00 F0

; RESET vector.
D_FFFC:
        .word L_F000                 ; $FFFC: 00 F0

; IRQ/BRK vector (hardware IRQ is not available on the 6507).
D_FFFE:
        .word L_F000                 ; $FFFE: 00 F0
