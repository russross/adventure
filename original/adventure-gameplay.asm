; Adventure: gameplay edition
;
; Routines explain the kingdom rules. Dot-prefixed labels stay within their routine;
; other labels name callable entries, shared entries, or referenced ROM data.
; The room names describe places in the map; their constants retain the original IDs.
; game_selection is 0, 2, or 4 for Games 1, 2, or 3. Object IDs are catalog offsets.
; Room/x/y records are shared by the player and movable inhabitants.
;
; Exact bytes and addressing modes are preserved. The address-labeled baseline is
; adventure.asm. Assemble this independent edition with make gameplay.

        processor 6502

; ROOMS AND OBJECTS

ROOM_GAME_SELECTION             = $00
ROOM_WEST_HALL                  = $01
ROOM_BELOW_GOLD_CASTLE          = $02
ROOM_SECRET_PASSAGE             = $03
ROOM_BLUE_MAZE_NORTH            = $04
ROOM_BLUE_MAZE_05               = $05
ROOM_BLUE_MAZE_SOUTH            = $06
ROOM_BLUE_MAZE_CENTER           = $07
ROOM_BLUE_MAZE_ENTRANCE         = $08
ROOM_DARK_CATACOMBS_CENTER      = $09
ROOM_DARK_CATACOMBS_ENTRANCE    = $0A
ROOM_DARK_CATACOMBS_SIDE        = $0B
ROOM_SIDE_PASSAGE_0C            = $0C
ROOM_SIDE_PASSAGE_0D            = $0D
ROOM_ABOVE_WHITE_CASTLE         = $0E
ROOM_WHITE_CASTLE               = $0F
ROOM_BLACK_CASTLE               = $10
ROOM_GOLD_CASTLE                = $11
ROOM_GOLD_TREASURY              = $12
ROOM_BLACK_MAZE_13              = $13
ROOM_BLACK_MAZE_14              = $14
ROOM_BLACK_MAZE_DOT             = $15
ROOM_BLACK_MAZE_ENTRANCE        = $16
ROOM_RED_MAZE_17                = $17
ROOM_RED_MAZE_NORTH             = $18
ROOM_RED_MAZE_SOUTH             = $19
ROOM_WHITE_CASTLE_INTERIOR      = $1A
ROOM_BLACK_CASTLE_INTERIOR      = $1B
ROOM_PURPLE_CHAMBER             = $1C
ROOM_RED_HALL                   = $1D
ROOM_ROBINETT_CREDITS           = $1E
OBJ_DARK_ROOM_LIGHT             = $00
OBJ_GOLD_GATE                   = $09
OBJ_WHITE_GATE                  = $12
OBJ_BLACK_GATE                  = $1B
OBJ_RHINDLE                     = $36
OBJ_YORGLE                      = $3F
OBJ_GRUNDLE                     = $48
OBJ_SWORD                       = $51
OBJ_BRIDGE                      = $5A
OBJ_GOLD_KEY                    = $63
OBJ_WHITE_KEY                   = $6C
OBJ_BLACK_KEY                   = $75
OBJ_EASTER_EGG_DOT              = $87
OBJ_NONE                        = $A2

; RECORD FIELDS, DRAGON STATES, AND SOUNDS

RECORD_ROOM                     = $00
RECORD_X                        = $01
RECORD_Y                        = $02
RECORD_DIRECTION                = $03
RECORD_STATE                    = $04
DRAGON_WANDERING                = $00
DRAGON_DEAD                     = $01
DRAGON_SWALLOWED                = $02
SOUND_VICTORY                   = $00
SOUND_ROAR                      = $01
SOUND_SWALLOW                   = $02
SOUND_DRAGON_SLAIN              = $03
SOUND_DROP                      = $04
SOUND_PICKUP                    = $05

; WORKING GAME STATE

room_image                      = $80
first_object_image              = $82
second_object_image             = $84
first_object_x                  = $86
first_object_y                  = $87
second_object_x                 = $88
second_object_y                 = $89
player_room                     = $8A
player_x                        = $8B
player_y                        = $8C
player_draw_y                   = $8D
draw_row                        = $8E
room_image_index                = $8F
first_object_row                = $90
second_object_row               = $91
previous_switches               = $92
record_pointer                  = $93
first_visible_object            = $95
second_visible_object           = $96
contact_object                  = $97
reserved_contact_byte           = $98
joystick_direction              = $99
saved_index                     = $9A
move_direction                  = $9B
last_display_candidate          = $9C
held_object                     = $9D
held_offset_x                   = $9E
held_offset_y                   = $9F
current_dragon                  = $A0
dot_room                        = $A1
rhindle_room                    = $A4
rhindle_state                   = $A8
yorgle_room                     = $A9
yorgle_state                    = $AD
grundle_room                    = $AE
grundle_state                   = $B2
magnet_room                     = $B3
magnet_y                        = $B5
sword_room                      = $B6
chalice_room                    = $B9
bridge_room                     = $BC
bridge_x                        = $BD
bridge_y                        = $BE
gold_key_room                   = $BF
white_key_room                  = $C2
black_key_room                  = $C5
gold_gate_state                 = $C8
white_gate_state                = $C9
black_gate_state                = $CA
bat_room                        = $CB
bat_x                           = $CC
bat_y                           = $CD
bat_direction                   = $CE
bat_animation                   = $CF
bat_carried_record              = $D0
bat_flight_timer                = $D1
reaction_list                   = $D2
creature_speed                  = $D4
creature_record                 = $D5
ignored_record                  = $D6
fire_history                    = $D7
scratch_value                   = $D8
light_room                      = $D9
light_x                         = $DA
light_y                         = $DB
image_state                     = $DC
game_selection                  = $DD
game_won                        = $DE
sound_remaining                 = $DF
sound_effect                    = $E0
reaction_index                  = $E1
safe_player_room                = $E2
safe_player_x                   = $E3
safe_player_y                   = $E4
idle_ticks                      = $E5
idle_periods                    = $E6

; DISPLAY AND INPUT REGISTERS

VSYNC                           = $00
VBLANK                          = $01
WSYNC                           = $02
NUSIZ0                          = $04
NUSIZ1                          = $05
COLUP0                          = $06
COLUP1                          = $07
COLUPF                          = $08
COLUBK                          = $09
CTRLPF                          = $0A
PF0                             = $0D
PF1                             = $0E
PF2                             = $0F
RESP0                           = $10
AUDC0                           = $15
AUDF0                           = $17
AUDV0                           = $19
AUDV1                           = $1A
GRP0                            = $1B
GRP1                            = $1C
ENAM0                           = $1D
ENAM1                           = $1E
ENABL                           = $1F
HMP0                            = $20
VDELP1                          = $26
HMOVE                           = $2A
HMCLR                           = $2B
CXCLR                           = $2C
CXP0FB                          = $32
CXP1FB                          = $33
CXM0FB                          = $34
CXM1FB                          = $35
CXBLPF                          = $36
CXPPMM                          = $37
INPT4                           = $3C
SWCHA                           = $0280
SWCHB                           = $0282
INTIM                           = $0284
TIM64T                          = $0296
RAM_START                       = $80

        org $F000

; Enter the normal startup path. The unused alternate entry remains below.
CartridgeEntry subroutine
        jmp     PowerOn

; Unused alternate startup bytes.
        .byte   $78,$D8,$4C,$06,$F3

; =============================================================================
; DRAW THE CURRENT ROOM
;
; The display draws the walls, the player square, and two selected objects.
; PrepareRoomDisplay chooses those objects; DrawRoom consumes that prepared view.
; Rows come in pairs. Object rows advance independently of the larger wall bands.
; =============================================================================


; Place the selected objects and the player before drawing the picture.
DrawRoom subroutine
        sta.b   HMCLR
        lda.b   first_object_x
        ldx     #$00
        jsr     PositionDrawable
        lda.b   second_object_x
        ldx     #$01
        jsr     PositionDrawable
        lda.b   player_x
        ldx     #$04
        jsr     PositionDrawable

; Apply their positions and begin collecting contacts for this picture.
        sta.b   WSYNC
        sta.b   HMOVE
        sta.b   CXCLR
        lda.b   player_y
        sec
        sbc     #$04
        sta.b   player_draw_y

; Wait until the picture can begin.
.wait_until_picture:
        lda.w   INTIM
        bne     .wait_until_picture

; Start all three row streams at the top; preload the first band of walls.
        lda     #$00
        sta.b   first_object_row
        sta.b   second_object_row
        sta.b   room_image_index
        sta.b   GRP1
        lda     #$01
        sta.b   VDELP1
        lda     #$68
        sta.b   draw_row

; Room walls advance in three-byte bands, independently of the object shapes.
        ldy.b   room_image_index
        lda     (room_image),y
        sta.b   PF0
        iny
        lda     (room_image),y
        sta.b   PF1
        iny
        lda     (room_image),y
        sta.b   PF2
        iny
        sty.b   room_image_index

; The picture begins here. The player remains a square rather than a bitmap.
        sta.b   WSYNC
        lda     #$00
        sta.b   VBLANK
        jmp     .first_object_row

; Advance the second object only after reaching its vertical position.
.next_row:
        lda.b   draw_row
        sec
        sbc.b   second_object_y
        sta.b   WSYNC
        bpl     .first_object_row
        ldy.b   second_object_row
        lda     (second_object_image),y
        sta.b   GRP1
        beq     .first_object_row
        inc.b   second_object_row

; Prepare the first object row. A zero holds the object blank thereafter.
.first_object_row:
        ldx     #$00
        lda.b   draw_row
        sec
        sbc.b   first_object_y
        bpl     .player_row
        ldy.b   first_object_row
        lda     (first_object_image),y
        tax
        beq     .player_row
        inc.b   first_object_row

; Enable the player square only for the rows around its vertical coordinate.
.player_row:
        ldy     #$00
        lda.b   draw_row
        sec
        sbc.b   player_draw_y
        and     #$FC
        bne     .room_row
        ldy     #$02

; A new wall band is needed every sixteen logical rows.
.room_row:
        lda.b   draw_row
        and     #$0F
        bne     .keep_walls

; Draw both objects and the player, then replace the wall band.
        sta.b   WSYNC
        sty.b   ENABL
        stx.b   GRP0
        ldy.b   room_image_index
        lda     (room_image),y
        sta.b   PF0
        iny
        lda     (room_image),y
        sta.b   PF1
        iny
        lda     (room_image),y
        sta.b   PF2
        iny
        sty.b   room_image_index

; Continue downward until the room picture is complete.
.advance_row:
        dec.b   draw_row
        lda.b   draw_row
        cmp     #$08
        bpl     .next_row
        sta.b   VBLANK
        jmp     .finish_picture

; Most rows reuse the walls already in place.
.keep_walls:
        sta.b   WSYNC
        sty.b   ENABL
        stx.b   GRP0
        jmp     .advance_row

; End the picture and leave the rest of the frame for game work.
.finish_picture:
        lda     #$00
        sta.b   GRP1
        sta.b   GRP0
        lda     #$20
        sta.w   TIM64T
        rts

; Position one drawable at horizontal coordinate A; X selects the drawable.
; The coarse delay and fine adjustment belong together and preserve pixel placement.
PositionDrawable subroutine
        ldy     #$02
        sec
.coarse_position:
        iny
        sbc     #$0F
        bcs     .coarse_position

; Keep the leftover coordinate as a fine adjustment.
        eor     #$FF
        sbc     #$06
        asl
        asl
        asl
        asl

; Align the positioning delay to a new line, then place the drawable.
        sty.b   WSYNC
.position_delay:
        dey
        bpl     .position_delay
        sta.b   RESP0,x
        sta.b   HMP0,x
        rts

; Finish the previous frame and begin the next one with the picture blanked.
BeginFrame subroutine
.wait_for_next_frame:
        lda.w   INTIM
        bne     .wait_for_next_frame

; Signal the new frame, then allow time for preparation before drawing.
        lda     #$02
        sta.b   WSYNC
        sta.b   VBLANK
        sta.b   WSYNC
        sta.b   WSYNC
        sta.b   WSYNC
        sta.b   VSYNC
        sta.b   WSYNC
        sta.b   WSYNC
        lda     #$00
        sta.b   WSYNC
        sta.b   VSYNC
        lda     #$2A
        sta.w   TIM64T
        rts

; =============================================================================
; DECIDE WHAT THE PLAYER SEES
;
; A room supplies walls, colors, side barriers, and exits. Its inhabitants supply
; their own positions and current images. Only two objects fit in one picture;
; selection rotates so crowded rooms flicker rather than permanently hiding items.
; =============================================================================


; Read the current room and choose its color or monochrome appearance.
PrepareRoomDisplay subroutine
        lda.b   player_room
        jsr     FindRoomRecord
        ldy     #$00
        lda     (record_pointer),y
        sta.b   room_image
        ldy     #$01
        lda     (record_pointer),y
        sta.b   room_image+1
        lda.w   SWCHB
        and     #$08
        beq     .monochrome_room
        ldy     #$02
        lda     (record_pointer),y
        jsr     ResolveDisplayColor
        sta.b   COLUPF
        jmp     .background_color
.monochrome_room:
        ldy     #$03
        lda     (record_pointer),y
        jsr     ResolveDisplayColor
        sta.b   COLUPF

; Set the background, wall behavior, and optional thin side barriers.
.background_color:
        lda     #$08
        jsr     ResolveDisplayColor
        sta.b   COLUBK
        ldy     #$04
        lda     (record_pointer),y
        sta.b   CTRLPF
        and     #$C0
        lsr
        lsr
        lsr
        lsr
        lsr
        sta.b   ENAM1
        lsr
        sta.b   ENAM0

; Choose two inhabitants. Prefer the second slot for the light or bridge;
; the light takes priority when both are present.
        jsr     SelectVisibleObjects
        lda.b   first_visible_object
        cmp     #$00
        beq     .swap_display_slots
        cmp     #$5A
        bne     .first_object
        lda.b   second_visible_object
        cmp     #$00
        beq     .first_object
.swap_display_slots:
        lda.b   first_visible_object
        sta.b   scratch_value
        lda.b   second_visible_object
        sta.b   first_visible_object
        lda.b   scratch_value
        sta.b   second_visible_object

; Copy the first inhabitant position, then select an image from its current state.
.first_object:
        ldx.b   first_visible_object
        lda.w   ObjectCatalog,x
        sta.b   record_pointer
        lda.w   ObjectCatalog+1,x
        sta.b   record_pointer+1
        ldy     #$01
        lda     (record_pointer),y
        sta.b   first_object_x
        ldy     #$02
        lda     (record_pointer),y
        sta.b   first_object_y

; Objects can keep their position and animation state in different records.
        lda.w   ObjectCatalog+2,x
        sta.b   record_pointer
        lda.w   ObjectCatalog+3,x
        sta.b   record_pointer+1
        ldy     #$00
        lda     (record_pointer),y
        sta.b   image_state
        lda.w   ObjectCatalog+4,x
        sta.b   record_pointer
        lda.w   ObjectCatalog+5,x
        sta.b   record_pointer+1

; The selected state range supplies this object image.
        jsr     SelectObjectImage
        iny
        lda     (record_pointer),y
        sta.b   first_object_image
        iny
        lda     (record_pointer),y
        sta.b   first_object_image+1

; Choose the first object color for the console color/monochrome setting.
        lda.w   SWCHB
        and     #$08
        beq     .monochrome_first_object
        lda.w   ObjectCatalog+6,x
        jsr     ResolveDisplayColor
        sta.b   COLUP0
        jmp     .first_object_size
.monochrome_first_object:
        lda.w   ObjectCatalog+7,x
        jsr     ResolveDisplayColor
        sta.b   COLUP0

; Apply its display size; the bridge and light use enlarged shapes.
.first_object_size:
        lda.w   ObjectCatalog+8,x
        ora     #$10
        sta.b   NUSIZ0

; Prepare the second inhabitant in the same way.
        ldx.b   second_visible_object
        lda.w   ObjectCatalog,x
        sta.b   record_pointer
        lda.w   ObjectCatalog+1,x
        sta.b   record_pointer+1
        ldy     #$01
        lda     (record_pointer),y
        sta.b   second_object_x
        ldy     #$02
        lda     (record_pointer),y
        sta.b   second_object_y

; Read the second object state before following its image-choice table.
        lda.w   ObjectCatalog+2,x
        sta.b   record_pointer
        lda.w   ObjectCatalog+3,x
        sta.b   record_pointer+1
        ldy     #$00
        lda     (record_pointer),y
        sta.b   image_state
        lda.w   ObjectCatalog+4,x
        sta.b   record_pointer
        lda.w   ObjectCatalog+5,x
        sta.b   record_pointer+1

; Keep its image and colors ready for DrawRoom.
        jsr     SelectObjectImage
        iny
        lda     (record_pointer),y
        sta.b   second_object_image
        iny
        lda     (record_pointer),y
        sta.b   second_object_image+1
        lda.w   SWCHB
        and     #$08
        beq     .monochrome_second_object
        lda.w   ObjectCatalog+6,x
        jsr     ResolveDisplayColor
        sta.b   COLUP1
        jmp     .second_object_size
.monochrome_second_object:
        lda.w   ObjectCatalog+7,x
        jsr     ResolveDisplayColor
        sta.b   COLUP1

; Finish the second object size and return the prepared picture.
.second_object_size:
        lda.w   ObjectCatalog+8,x
        ora     #$10
        sta.b   NUSIZ1
        rts

; Scan the object catalog cyclically, filling at most two slots in this room.
; Resume at the last candidate next time so every inhabitant gets a turn.
SelectVisibleObjects subroutine
        ldy.b   last_display_candidate
        lda     #OBJ_NONE
        sta.b   first_visible_object
        sta.b   second_visible_object
.next_candidate:
        tya
        clc
        adc     #$09
        cmp     #OBJ_NONE
        bcc     .read_candidate
        lda     #$00

; The first byte of an object position record is its room.
.read_candidate:
        tay
        lda.w   ObjectCatalog,y
        sta.b   record_pointer
        lda.w   ObjectCatalog+1,y
        sta.b   record_pointer+1
        ldx     #$00
        lda     (record_pointer,x)
        cmp.b   player_room
        bne     .check_scan_complete

; Fill the first empty display slot, then stop once the second is filled.
        lda.b   first_visible_object
        cmp     #OBJ_NONE
        bne     .fill_second_slot
        sty.b   first_visible_object
        jmp     .check_scan_complete
.fill_second_slot:
        sty.b   second_visible_object
        jmp     .remember_scan_position

; A full trip around the catalog also ends the search.
.check_scan_complete:
        cpy.b   last_display_candidate
        bne     .next_candidate
.remember_scan_position:
        sty.b   last_display_candidate
        rts

; Locate a room record from its room number in A. Each record is nine bytes.
FindRoomRecord subroutine
        sta.b   scratch_value
        sta.b   record_pointer
        lda     #$00
        sta.b   record_pointer+1

; Build the eight-times part of the record offset.
        clc
        rol.b   record_pointer
        rol.b   record_pointer+1
        rol.b   record_pointer
        rol.b   record_pointer+1
        rol.b   record_pointer
        rol.b   record_pointer+1

; Add the original room number to make nine times its value.
        lda.b   scratch_value
        clc
        adc.b   record_pointer
        sta.b   record_pointer
        lda     #$00
        adc.b   record_pointer+1
        sta.b   record_pointer+1

; Add the room table base to produce the final pointer.
        lda     #<RoomTable
        clc
        adc.b   record_pointer
        sta.b   record_pointer
        lda     #>RoomTable
        adc.b   record_pointer+1
        sta.b   record_pointer+1
        rts

; Choose the first image range that includes the current object state.
SelectObjectImage subroutine
        ldy     #$00
        lda.b   image_state
.next_image_range:
        cmp     (record_pointer),y
        bcc     .image_found
        beq     .image_found
        iny
        iny
        iny
        jmp     .next_image_range
.image_found:
        rts

; After prolonged inactivity the display changes colors. Any control activity
; clears that condition; the low counter also seeds Game 3 placement.
TrackInactivity subroutine
        inc.b   idle_ticks
        bne     .check_controls
        inc.b   idle_periods
        bne     .check_controls
        lda     #$80
        sta.b   idle_periods

; Treat either joystick movement or a console button as activity.
.check_controls:
        lda.w   SWCHA
        cmp     #$FF
        bne     .activity_detected
        lda.w   SWCHB
        and     #$03
        cmp     #$03
        beq     .done
.activity_detected:
        lda     #$00
        sta.b   idle_periods
.done:
        rts

; Some colors are literal; others refer to a value in working memory.
; Apply the inactivity color variation after resolving that choice.
ResolveDisplayColor subroutine
        lsr
        bcc     .apply_idle_colors
        tay
        lda.w   RAM_START,y
.apply_idle_colors:
        ldy.b   idle_periods
        bpl     .restore_color_scale
        eor.b   idle_periods
        and     #$FB
.restore_color_scale:
        asl
        rts

; Find the room/x/y record for object catalog offset X.
FindObjectPosition subroutine
        lda.w   ObjectCatalog,x
        sta.b   record_pointer
        lda.w   ObjectCatalog+1,x
        sta.b   record_pointer+1
        rts

; =============================================================================
; STARTUP AND THE GAME TURN
;
; The cartridge initializes its display and working state, then runs forever.
; One turn spans three pictures: player motion and creature updates are spread
; across them, while the prepared room picture remains available throughout.
; =============================================================================


; Initialize the display, call stack, and game memory once at power-on.
PowerOn subroutine
        sei
        cld
        ldx     #$28
        lda     #$00
.clear_display:
        sta.b   NUSIZ0,x
        dex
        bpl     .clear_display
        txs

; Clear working memory before choosing the first world.
.clear_game_state:
        sta.b   $00,x
        dex
        bmi     .clear_game_state

; Set the side barriers and initialize the game-selection world.
        jsr     PositionSideBarriers
        jsr     InitializeWorld

; Process controls and sound before deciding whether normal play continues.
RunGame subroutine
        jsr     PollConsoleSwitches
        jsr     PlayGameSound
        jsr     TrackInactivity
        lda.b   game_won
        bne     .victory_frames

; The chalice wins the game as soon as its room is the Golden Castle treasury.
; Its carrier and precise position do not enter this test.
        lda.b   chalice_room
        cmp     #ROOM_GOLD_TREASURY
        bne     .play_turn
        lda     #$FF
        sta.b   sound_remaining
        sta.b   game_won
        lda     #SOUND_VICTORY
        sta.b   sound_effect

; First picture: sample both movement axes, move the carried item, prepare
; the room, draw it, then act on pickup/drop contacts from that picture.
.play_turn:
        ldy     #$00
        jsr     MovePlayer
        jsr     MoveHeldObject
        jsr     BeginFrame
        jsr     PrepareRoomDisplay
        jsr     DrawRoom
        jsr     HandlePickupAndDrop

; Second picture: resolve vertical movement and dark-room lighting.
; The bat and gates update before drawing; Grundle and Yorgle follow it.
        ldy     #$01
        jsr     MovePlayer
        jsr     UpdateDarkRoomLight
        jsr     BeginFrame
        jsr     UpdateBat
        jsr     UpdateCastleGates
        jsr     DrawRoom
        jsr     UpdateGrundle
        jsr     UpdateYorgle

; Third picture: resolve horizontal movement, then update Rhindle and the magnet.
; Draw the prepared room again and begin the next turn.
        jsr     BeginFrame
        ldy     #$02
        jsr     MovePlayer
        jsr     UpdateRhindle
        jsr     UpdateMagnet
        jsr     DrawRoom
        jmp     RunGame

; After victory, keep displaying the scene and advancing the color celebration.
.victory_frames:
        jsr     BeginFrame
        jsr     DrawRoom
        jsr     PrepareRoomDisplay
        jmp     RunGame

; Put the thin room barriers at the fixed left and right edges of the picture.
PositionSideBarriers subroutine
        lda     #$0D
        ldx     #$02
        jsr     PositionDrawable
        lda     #$96
        ldx     #$03
        jsr     PositionDrawable
        sta.b   WSYNC
        sta.b   HMOVE
        rts

; =============================================================================
; RESET, GAME SELECTION, AND STARTING POSITIONS
;
; During play, RESET returns the player to the Golden Castle and revives dragons
; without restoring items. After victory it rebuilds the selected starting world.
; SELECT prepares a starting world and shows its game number.
; Game 3 starts from Game 2 positions, then randomizes selected object rooms.
; =============================================================================


; Respond to newly pressed console buttons rather than a held button.
PollConsoleSwitches subroutine
        lda.w   SWCHB
        eor     #$FF
        and.b   previous_switches
        and     #$01
        beq     .check_select
; After victory, RESET restores the complete world rather than just the player.
        lda.b   game_won
        cmp     #$FF
        beq     InitializeWorld

; RESET during play restores the player outside the Golden Castle.
        lda     #ROOM_GOLD_CASTLE
        sta.b   player_room
        sta.b   safe_player_room
        lda     #$50
        sta.b   player_x
        sta.b   safe_player_x
        lda     #$20
        sta.b   player_y
        sta.b   safe_player_y

; Revive all three dragons, stop the current sound, and release the held object.
; The scattered items retain their positions.
        lda     #$00
        sta.b   rhindle_state
        sta.b   yorgle_state
        sta.b   grundle_state
        sta.b   sound_remaining
        lda     #OBJ_NONE
        sta.b   held_object

; SELECT opens the game-number room. There it cycles through Games 1, 2, and 3.
.check_select:
        lda.w   SWCHB
        eor     #$FF
        and.b   previous_switches
        and     #$02
        beq     RememberConsoleSwitches
        lda.b   player_room
        cmp     #$00
        bne     InitializeWorld

; The selection is stored as 0, 2, or 4, matching the starting-world pointer pairs.
        lda.b   game_selection
        clc
        adc     #$02
        cmp     #$06
        bcc     .save_selection
        lda     #$00
.save_selection:
        sta.b   game_selection

; Hide the player in the game-number room and restore the selected starting world.
InitializeWorld subroutine
        lda     #$00
        sta.b   player_room
        sta.b   safe_player_room
        lda     #$00
        sta.b   player_y
        sta.b   safe_player_y

; Games 2 and 3 share the same initial image of object memory.
        ldy.b   game_selection
        lda.w   StartingWorlds,y
        sta.b   record_pointer
        lda.w   StartingWorlds+1,y
        sta.b   record_pointer+1
        ldy     #$30

; Copy all movable records, dragon states, gate states, and bat bookkeeping.
.copy_starting_world:
        lda     (record_pointer),y
        sta.w   dot_room,y
        dey
        bpl     .copy_starting_world

; Only Game 3 replaces those fixed rooms with randomized placements.
        lda.b   game_selection
        cmp     #$04
        bcc     .world_ready
        jsr     RandomizeGameThree
        jsr     BeginFrame
        jsr     DrawRoom

; Clear victory and begin with nothing carried.
.world_ready:
        lda     #$00
        sta.b   game_won
        lda     #OBJ_NONE
        sta.b   held_object

; This shared tail also finishes console polling without rebuilding the world.
RememberConsoleSwitches subroutine
        lda.w   SWCHB
        sta.b   previous_switches
        rts

; Randomize eleven movable objects, keeping each inside its permitted room range.
; The chalice, White Key, and Black Key have tighter placement constraints.
RandomizeGameThree subroutine
        ldy     #$1E

; Generate another room until both the lower and upper bounds accept it.
.try_room:
        lda.b   idle_ticks
        lsr
        lsr
        lsr
        lsr
        lsr
        sec
        adc.b   idle_ticks
        sta.b   idle_ticks
        and     #$1F
        cmp.w   RandomRoomBounds+1,y
        bcc     .try_room
        cmp.w   RandomRoomBounds+2,y
        beq     .accept_room
        bcs     .try_room

; Store the room in that object record, then proceed to the next constraint.
.accept_room:
        ldx.w   RandomRoomBounds,y
        sta.b   RECORD_ROOM,x
        dey
        dey
        dey
        bpl     .try_room
        rts

; Game 3 room bounds: object record, first permitted room, last permitted room.
RandomRoomBounds:
        .byte   chalice_room,ROOM_BLACK_MAZE_13,ROOM_WHITE_CASTLE_INTERIOR
        .byte   rhindle_room,ROOM_WEST_HALL,ROOM_RED_HALL
        .byte   yorgle_room,ROOM_WEST_HALL,ROOM_RED_HALL
        .byte   grundle_room,ROOM_WEST_HALL,ROOM_RED_HALL
        .byte   sword_room,ROOM_WEST_HALL,ROOM_RED_HALL
        .byte   bridge_room,ROOM_WEST_HALL,ROOM_RED_HALL
        .byte   gold_key_room,ROOM_WEST_HALL,ROOM_RED_HALL
        .byte   white_key_room,ROOM_WEST_HALL,ROOM_BLACK_MAZE_ENTRANCE
        .byte   black_key_room,ROOM_WEST_HALL,ROOM_GOLD_TREASURY
        .byte   bat_room,ROOM_WEST_HALL,ROOM_RED_HALL
        .byte   magnet_room,ROOM_WEST_HALL,ROOM_RED_HALL

; Game 3 first copies Game 2 positions, then randomizes them.
StartingWorlds:
        .word   GameOneWorld
        .word   GameTwoWorld
        .word   GameTwoWorld

; Fixed starting world for Game 1.
GameOneWorld:
        .byte   ROOM_BLACK_MAZE_DOT,$51,$12                      ; Easter egg dot
        .byte   ROOM_ABOVE_WHITE_CASTLE,$50,$20,$00,DRAGON_WANDERING ; Rhindle (red dragon)
        .byte   ROOM_WEST_HALL,$50,$20,$00,DRAGON_WANDERING      ; Yorgle (yellow dragon)
        .byte   ROOM_RED_HALL,$50,$20,$00,DRAGON_WANDERING       ; Grundle (green dragon)
        .byte   ROOM_BLACK_CASTLE_INTERIOR,$80,$20               ; Magnet
        .byte   ROOM_GOLD_TREASURY,$20,$20                       ; Sword
        .byte   ROOM_PURPLE_CHAMBER,$30,$20                      ; Enchanted chalice
        .byte   ROOM_BLUE_MAZE_NORTH,$29,$37                     ; Bridge
        .byte   ROOM_GOLD_CASTLE,$20,$40                         ; Gold Key
        .byte   ROOM_ABOVE_WHITE_CASTLE,$20,$40                  ; White Key
        .byte   ROOM_RED_HALL,$20,$40                            ; Black Key
        .byte   $1C,$1C,$1C                                      ; Castle gate states
        .byte   ROOM_WHITE_CASTLE_INTERIOR,$20,$20,$00,$00       ; Black bat
        .byte   $78,$00                                          ; Bat cargo/timer

; Fixed starting world for Game 2; Game 3 randomizes its room assignments.
GameTwoWorld:
        .byte   ROOM_BLACK_MAZE_DOT,$51,$12                      ; Easter egg dot
        .byte   ROOM_BLACK_MAZE_14,$50,$20,$A0,DRAGON_WANDERING  ; Rhindle (red dragon)
        .byte   ROOM_RED_MAZE_SOUTH,$50,$20,$A0,DRAGON_WANDERING ; Yorgle (yellow dragon)
        .byte   ROOM_BLUE_MAZE_NORTH,$50,$20,$A0,DRAGON_WANDERING ; Grundle (green dragon)
        .byte   ROOM_ABOVE_WHITE_CASTLE,$80,$20                  ; Magnet
        .byte   ROOM_GOLD_CASTLE,$20,$20                         ; Sword
        .byte   ROOM_BLACK_MAZE_14,$30,$20                       ; Enchanted chalice
        .byte   ROOM_DARK_CATACOMBS_SIDE,$40,$40                 ; Bridge
        .byte   ROOM_DARK_CATACOMBS_CENTER,$20,$40               ; Gold Key
        .byte   ROOM_BLUE_MAZE_SOUTH,$20,$40                     ; White Key
        .byte   ROOM_RED_MAZE_SOUTH,$20,$40                      ; Black Key
        .byte   $1C,$1C,$1C                                      ; Castle gate states
        .byte   ROOM_BELOW_GOLD_CASTLE,$20,$20,$90,$00           ; Black bat
        .byte   $78,$00                                          ; Bat cargo/timer

; =============================================================================
; THE PLAYER, THE BRIDGE, AND CARRIED OBJECTS
;
; The previous picture reports which walls or objects touched the player.
; Movement retries from the last safe position when blocked. The bridge creates
; a narrow vertical exception; items are picked up by contact and dropped with fire.
; =============================================================================


; Check the last picture for walls and side barriers that block the player.
MovePlayer subroutine
        lda.b   CXBLPF
        and     #$80
        bne     .blocked
        lda.b   CXM0FB
        and     #$40
        bne     .blocked

; Ignore the thin right wall when the dot occupies the second display slot.
        lda.b   CXM1FB
        and     #$40
        beq     .check_first_object
        lda.b   second_visible_object
        cmp     #OBJ_EASTER_EGG_DOT
        bne     .blocked

; The dark-room light is not a solid obstacle; other displayed objects are.
.check_first_object:
        lda.b   CXP0FB
        and     #$40
        beq     .check_second_object
        lda.b   first_visible_object
        cmp     #OBJ_DARK_ROOM_LIGHT
        bne     .blocked
.check_second_object:
        lda.b   CXP1FB
        and     #$40
        beq     .accept_position
        lda.b   second_visible_object
        cmp     #OBJ_DARK_ROOM_LIGHT
        bne     .blocked
        jmp     .accept_position

; A blocked move normally rolls back. On the horizontal correction phase,
; standing in the bridge corridor can still validate the player position.
.blocked:
        cpy     #$02
        bne     .sample_input
        lda.b   held_object
        cmp     #OBJ_BRIDGE
        beq     .sample_input

; The bridge must be in this room, with the player between its vertical rails.
        lda.b   player_room
        cmp.b   bridge_room
        bne     .sample_input
        lda.b   player_x
        sec
        sbc.b   bridge_x
        cmp     #$0A
        bcc     .sample_input
        cmp     #$17
        bcs     .sample_input

; The player must also overlap the bridge height. Carrying it disables this exception.
        lda.b   bridge_y
        sec
        sbc.b   player_y
        cmp     #$FC
        bcs     .accept_position
        cmp     #$19
        bcs     .sample_input

; Accept the position as safe, then prepare movement from that saved position.
.accept_position:
        lda     #$FF
        sta.b   joystick_direction
        lda.b   player_room
        sta.b   safe_player_room
        lda.b   player_x
        sta.b   safe_player_x
        lda.b   player_y
        sta.b   safe_player_y

; Sample the joystick on the first phase; later phases reuse that reading.
.sample_input:
        cpy     #$00
        bne     .retry_from_safe_position
        lda.w   SWCHA
        sta.b   joystick_direction

; Retry from the last accepted room and coordinates.
.retry_from_safe_position:
        lda.b   safe_player_room
        sta.b   player_room
        lda.b   safe_player_x
        sta.b   player_x
        lda.b   safe_player_y
        sta.b   player_y

; The phase mask suppresses selected axes; three small steps apply the movement.
        lda.b   joystick_direction
        ora.w   PlayerMovementMasks,y
        sta.b   move_direction
        ldy     #$03
        ldx     #player_room
        jsr     MoveObjectThroughWorld
        rts

; First phase allows both axes; the next phases correct vertical and horizontal motion.
PlayerMovementMasks:
        .byte   $00,$C0,$30

; A new fire press releases the carried object and plays the drop sound.
HandlePickupAndDrop subroutine
        rol.b   INPT4
        ror.b   fire_history
        lda.b   fire_history
        and     #$C0
        cmp     #$40
        bne     .check_contact
        lda     #OBJ_NONE
        cmp.b   held_object
        beq     .check_contact
        sta.b   held_object
        lda     #SOUND_DROP
        sta.b   sound_effect
        lda     #$04
        sta.b   sound_remaining

; Find a displayed object touching the player. Either display slot can report it.
.check_contact:
        lda     #$FF
        sta.b   reserved_contact_byte
        lda.b   CXP0FB
        and     #$40
        beq     .check_second_object
        lda.b   first_visible_object
        sta.b   contact_object
        jmp     .check_pickup
.check_second_object:
        lda.b   CXP1FB
        and     #$40
        beq     .nothing_to_pick_up
        lda.b   second_visible_object
        sta.b   contact_object
        jmp     .check_pickup
.nothing_to_pick_up:
        jmp     .done

; Ignore gates, scenery, and contacts from a different room; portable objects
; begin with the sword in the catalog.
.check_pickup:
        ldx.b   contact_object
        jsr     FindObjectPosition
        lda.b   contact_object
        cmp     #OBJ_SWORD
        bcc     .done
        ldy     #$00
        lda     (record_pointer),y
        cmp.b   player_room
        bne     .done

; A new item replaces the held object and makes the pickup sound.
        lda.b   contact_object
        cmp.b   held_object
        beq     .attach_object
        lda     #SOUND_PICKUP
        sta.b   sound_effect
        lda     #$04
        sta.b   sound_remaining

; Push the contacted object ahead, then keep its offset from the player.
; That offset determines how it follows the player on later turns.
.attach_object:
        lda.b   contact_object
        sta.b   held_object
        ldx.b   record_pointer
        ldy     #$06
        lda.b   joystick_direction
        jsr     StepObject

; Remember both axes of the attachment.
        ldy     #$01
        lda     (record_pointer),y
        sec
        sbc.b   player_x
        sta.b   held_offset_x
        ldy     #$02
        lda     (record_pointer),y
        sec
        sbc.b   player_y
        sta.b   held_offset_y
.done:
        rts

; Move the held object to the player room and keep its attachment offset.
MoveHeldObject subroutine
        ldx.b   held_object
        cpx     #OBJ_NONE
        beq     .done
        jsr     FindObjectPosition
        ldy     #$00
        lda.b   player_room
        sta     (record_pointer),y

; Update the carried x/y position before checking exits and gate passage.
        ldy     #$01
        lda.b   player_x
        clc
        adc.b   held_offset_x
        sta     (record_pointer),y
        ldy     #$02
        lda.b   player_y
        clc
        adc.b   held_offset_y
        sta     (record_pointer),y

; Even without a directional step, a carried object can cross a room boundary.
        ldy     #$00
        lda     #$FF
        ldx.b   record_pointer
        jsr     MoveObjectThroughWorld
.done:
        rts

; =============================================================================
; MOVEMENT THROUGH THE KINGDOM
;
; Player and movable objects share room/x/y records and the same edge handling.
; Stepping off a room chooses one of its exits. Open gates connect their castle
; interiors to the outside. Some exits differ between Game 1 and Games 2/3.
; =============================================================================


; Apply the requested steps to the record at X, then check castle exits.
MoveObjectThroughWorld subroutine
        jsr     StepObject
        ldy     #$02

; A gate at its closed state blocks the special interior-to-outside exit.
.check_gate:
        sty.b   saved_index
        lda.w   gold_gate_state,y
        cmp     #$1C
        beq     .next_gate
        ldy.b   saved_index
        lda.b   RECORD_ROOM,x
        cmp.w   CastleInteriorRooms,y
        bne     .next_gate

; Crossing the bottom of an open castle interior puts the object outside.
; The gate is held open while that transition completes.
        lda.b   RECORD_Y,x
        cmp     #$0D
        bpl     .next_gate
        lda.w   CastleOutsideRooms,y
        sta.b   RECORD_ROOM,x
        lda     #$50
        sta.b   RECORD_X,x
        lda     #$2C
        sta.b   RECORD_Y,x
        lda     #$01
        sta.w   gold_gate_state,y
        rts

; If no castle exit applies, handle the four ordinary screen edges.
.next_gate:
        ldy.b   saved_index
        dey
        bpl     .check_gate

; Cross the top edge and wrap to the bottom of the room above.
        lda.b   RECORD_Y,x
        cmp     #$6A
        bmi     .check_left_edge
        lda     #$0D
        sta.b   RECORD_Y,x
        ldy     #$05
        jmp     .take_room_exit

; Cross the left edge and wrap to the right of the room to the left.
.check_left_edge:
        lda.b   RECORD_X,x
        cmp     #$03
        bcc     .wrap_left
        cmp     #$F0
        bcs     .wrap_left
        jmp     .check_bottom_edge

; The player square and object images have slightly different rightmost positions.
.wrap_left:
        cpx     #$8A
        beq     .wrap_player_left
        lda     #$9A
        jmp     .set_left_wrap
.wrap_player_left:
        lda     #$9E
.set_left_wrap:
        sta.b   RECORD_X,x
        ldy     #$08
        jmp     .take_room_exit

; Cross the bottom edge and wrap to the top of the room below.
.check_bottom_edge:
        lda.b   RECORD_Y,x
        cmp     #$0D
        bcs     .check_right_edge
        lda     #$69
        sta.b   RECORD_Y,x
        ldy     #$07
        jmp     .take_room_exit

; At the right edge, the player has an additional secret-room test.
.check_right_edge:
        lda.b   RECORD_X,x
        cpx     #player_room
        bne     .check_object_right_edge
        cmp     #$9F
        bcc     .done

; With the dot removed from its hiding room, this passage right-edge exit leads
; to Warren Robinett credits. The wall check still controls reaching that edge.
        lda.b   RECORD_ROOM,x
        cmp     #ROOM_SECRET_PASSAGE
        bne     .wrap_right
        lda.b   dot_room
        cmp     #ROOM_BLACK_MAZE_DOT
        beq     .wrap_right
        lda     #ROOM_ROBINETT_CREDITS
        sta.b   RECORD_ROOM,x
        lda     #$03
        sta.b   RECORD_X,x
        jmp     .done

; Other objects use their ordinary right-edge wrap.
.check_object_right_edge:
        cmp     #$9B
        bcc     .done
.wrap_right:
        lda     #$03
        sta.b   RECORD_X,x
        ldy     #$06
        jmp     .take_room_exit

; Use the chosen exit from this room and resolve any game-dependent destination.
.take_room_exit:
        lda.b   RECORD_ROOM,x
        jsr     FindRoomRecord
        lda     (record_pointer),y
        jsr     ResolveGameExit
        sta.b   RECORD_ROOM,x
.done:
        rts

; Apply Y unit steps using active-low direction bits in A to the record at X.
; Coordinates increase rightward and upward; opposing directions cancel.
StepObject subroutine
        sta.b   move_direction
.next_step:
        dey
        bmi     .done
        lda.b   move_direction
        and     #$80
        bne     .left
        inc.b   RECORD_X,x

; The remaining direction bits independently handle left, up, and down.
.left:
        lda.b   move_direction
        and     #$40
        bne     .up
        dec.b   RECORD_X,x
.up:
        lda.b   move_direction
        and     #$10
        bne     .down
        inc.b   RECORD_Y,x

; Repeat until the requested movement distance has been exhausted.
.down:
        lda.b   move_direction
        and     #$20
        bne     .step_complete
        dec.b   RECORD_Y,x
.step_complete:
        jmp     .next_step
.done:
        rts

; Most exits are direct room numbers. Tagged exits index three destinations,
; one for each selected game.
ResolveGameExit subroutine
        cmp     #$80
        bcc     .done
        sec
        sbc     #$80
        sta.b   scratch_value
        lda.b   game_selection
        lsr
        clc
        adc.b   scratch_value
        tay
        lda.w   GameDependentExits,y
.done:
        rts

; =============================================================================
; CONTACT AND CREATURE REACTIONS
;
; Interactions use the two objects in the displayed picture. An object that was
; not selected cannot report a contact in that picture, explaining why crowded
; rooms can make the sword and other interactions seem intermittent.
; =============================================================================


; Return whether catalog offset A touched the player in the last picture.
IsTouchingPlayer subroutine
        cmp.b   first_visible_object
        beq     .first_display_slot
        cmp.b   second_visible_object
        beq     .second_display_slot
        lda     #$00
        rts
.first_display_slot:
        lda.b   CXP0FB
        and     #$40
        rts
.second_display_slot:
        lda.b   CXP1FB
        and     #$40
        rts

; For catalog offset X, return the other displayed object touching it.
; Return OBJ_NONE when the pair did not collide or X was not displayed.
FindTouchingObject subroutine
        lda.b   CXPPMM
        and     #$80
        beq     .no_contact
        cpx.b   first_visible_object
        beq     .return_second_object
        cpx.b   second_visible_object
        beq     .return_first_object
.no_contact:
        lda     #OBJ_NONE
        rts
.return_second_object:
        lda.b   second_visible_object
        rts
.return_first_object:
        lda.b   first_visible_object
        rts

; Choose a reaction and move the creature. Without an eligible reaction,
; keep its previous direction rather than stopping it.
MoveCreature subroutine
        jsr     ChooseReaction
        ldx.b   creature_record
        lda.b   move_direction
        bne     .apply_direction
        lda.b   RECORD_DIRECTION,x
.apply_direction:
        sta.b   RECORD_DIRECTION,x
        ldy.b   creature_speed
        jsr     MoveObjectThroughWorld
        rts

; Read ordered pairs from reaction_list. Both records must share a room,
; and neither may be the currently ignored record.
ChooseReaction subroutine
        lda     #$00
        sta.b   reaction_index
.next_pair:
        ldy.b   reaction_index
        lda     (reaction_list),y
        tax
        iny
        lda     (reaction_list),y
        tay

; Pair order determines attraction or avoidance when its direction is applied
; to a different creature: sword-to-dragon makes the dragon flee the sword.
        lda.b   RECORD_ROOM,x
        cmp.w   RECORD_ROOM,y
        bne     .skip_pair
        cpy.b   ignored_record
        beq     .skip_pair
        cpx.b   ignored_record
        beq     .skip_pair
        jsr     DirectionBetweenObjects
        rts

; The first eligible pair wins. A zero marks the end of the priorities.
.skip_pair:
        inc.b   reaction_index
        inc.b   reaction_index
        ldy.b   reaction_index
        lda     (reaction_list),y
        bne     .next_pair
        lda     #$00
        sta.b   move_direction
        rts

; Construct the direction from record X toward record Y.
; Equal coordinates leave that axis idle.
DirectionBetweenObjects subroutine
        lda     #$FF
        sta.b   move_direction
        lda.w   RECORD_ROOM,y
        cmp.b   RECORD_ROOM,x
        bne     .done
        lda.w   RECORD_X,y
        cmp.b   RECORD_X,x
        bcc     .toward_left
        beq     .vertical_direction
        lda.b   move_direction
        and     #$7F
        sta.b   move_direction
        jmp     .vertical_direction

; Combine the chosen horizontal direction with the independent vertical direction.
.toward_left:
        lda.b   move_direction
        and     #$BF
        sta.b   move_direction
.vertical_direction:
        lda.w   RECORD_Y,y
        cmp.b   RECORD_Y,x
        bcc     .toward_down
        beq     .done
        lda.b   move_direction
        and     #$EF
        sta.b   move_direction
        jmp     .done

; Return the combined movement mask.
.toward_down:
        lda.b   move_direction
        and     #$DF
        sta.b   move_direction
.done:
        lda.b   move_direction
        rts

; =============================================================================
; RHINDLE, YORGLE, AND GRUNDLE
;
; Each dragon has an ordered reaction list. The first eligible pair in one room
; wins: fleeing the sword takes precedence over chasing the player or guarding
; treasure. Rhindle moves faster than Yorgle and Grundle.
; =============================================================================


; Rhindle, the red dragon, moves three steps and guards the White Key.
UpdateRhindle subroutine
        lda     #<RhindleReactions
        sta.b   reaction_list
        lda     #>RhindleReactions
        sta.b   reaction_list+1
        lda     #$03
        sta.b   creature_speed
        ldx     #OBJ_RHINDLE
        jsr     UpdateDragon
        rts

; Rhindle flees the sword, then pursues the player, chalice, and White Key.
RhindleReactions:
        .byte   sword_room,rhindle_room
        .byte   rhindle_room,player_room
        .byte   rhindle_room,chalice_room
        .byte   rhindle_room,white_key_room
        .byte   $00                                              ; End of priorities

; Yorgle, the yellow dragon, moves two steps and fears the Gold Key.
UpdateYorgle subroutine
        lda     #<YorgleReactions
        sta.b   reaction_list
        lda     #>YorgleReactions
        sta.b   reaction_list+1
        lda     #$02
        sta.b   creature_speed
        ldx     #OBJ_YORGLE
        jsr     UpdateDragon
        rts

; Yorgle flees the sword and Gold Key before pursuing the player and chalice.
YorgleReactions:
        .byte   sword_room,yorgle_room
        .byte   gold_key_room,yorgle_room
        .byte   yorgle_room,player_room
        .byte   yorgle_room,chalice_room
        .byte   $00                                              ; End of priorities

; Grundle, the green dragon, moves two steps and guards the bridge, magnet,
; and Black Key after pursuing the player and chalice.
UpdateGrundle subroutine
        lda     #<GrundleReactions
        sta.b   reaction_list
        lda     #>GrundleReactions
        sta.b   reaction_list+1
        lda     #$02
        sta.b   creature_speed
        ldx     #OBJ_GRUNDLE
        jsr     UpdateDragon
        rts

; Grundle flees the sword, then pursues the player, chalice, bridge, magnet, and Black Key.
GrundleReactions:
        .byte   sword_room,grundle_room
        .byte   grundle_room,player_room
        .byte   grundle_room,chalice_room
        .byte   grundle_room,bridge_room
        .byte   grundle_room,magnet_room
        .byte   grundle_room,black_key_room
        .byte   $00                                              ; End of priorities

; A wandering dragon follows its priorities, roars on player contact,
; and can be slain by touching the sword. Other states handle the bite.
UpdateDragon subroutine
        stx.b   current_dragon
        lda.w   ObjectCatalog,x
        tax
        lda.b   RECORD_STATE,x
        cmp     #DRAGON_WANDERING
        bne     .other_dragon_states

; The difficulty setting can remove the sword from its avoidance priorities.
        lda.w   SWCHB
        and     #$80
        beq     .ignore_sword
        lda     #$00
        jmp     .move_dragon
.ignore_sword:
        lda     #sword_room
.move_dragon:
        sta.b   ignored_record
        stx.b   creature_record
        jsr     MoveCreature

; Touching the player starts a bite delay chosen by game and difficulty.
        lda.b   current_dragon
        jsr     IsTouchingPlayer
        beq     .check_sword
        lda.w   SWCHB
        rol
        rol
        rol
        and     #$01
        ora.b   game_selection
        tay
        lda.w   DragonBiteDelays,y
        sta.b   RECORD_STATE,x

; Begin the lunge at the player last accepted position and make the roar sound.
        lda.b   safe_player_x
        sta.b   RECORD_X,x
        lda.b   safe_player_y
        sta.b   RECORD_Y,x
        lda     #SOUND_ROAR
        sta.b   sound_effect
        lda     #$10
        sta.b   sound_remaining

; Sword contact changes the dragon to its dead state and plays the death sound.
.check_sword:
        stx.b   saved_index
        ldx.b   current_dragon
        jsr     FindTouchingObject
        ldx.b   saved_index
        cmp     #OBJ_SWORD
        bne     .finish_living_dragon
        lda     #DRAGON_DEAD
        sta.b   RECORD_STATE,x
        lda     #SOUND_DRAGON_SLAIN
        sta.b   sound_effect
        lda     #$10
        sta.b   sound_remaining
.finish_living_dragon:
        jmp     .done

; Dead dragons stay still. A dragon that swallowed the player carries them
; inside its body instead of applying normal player movement here.
.other_dragon_states:
        cmp     #DRAGON_DEAD
        beq     .done
        cmp     #DRAGON_SWALLOWED
        bne     .advance_bite

; Keep the player attached to the swallowing dragon.
        lda.b   RECORD_ROOM,x
        sta.b   player_room
        sta.b   safe_player_room
        lda.b   RECORD_X,x
        clc
        adc     #$03
        sta.b   player_x
        sta.b   safe_player_x
        lda.b   RECORD_Y,x
        sec
        sbc     #$0A
        sta.b   player_y
        sta.b   safe_player_y
        jmp     .done

; During the roar/bite countdown, the player can escape the contact.
; After the countdown, continued contact completes the swallow.
.advance_bite:
        inc.b   RECORD_STATE,x
        lda.b   RECORD_STATE,x
        cmp     #$FC
        bcc     .done
        lda.b   current_dragon
        jsr     IsTouchingPlayer
        beq     .done

; Enter the swallowed state and trigger its sound.
        lda     #DRAGON_SWALLOWED
        sta.b   RECORD_STATE,x
        lda     #SOUND_SWALLOW
        sta.b   sound_effect
        lda     #$10
        sta.b   sound_remaining

; Keep the swallowed player inside the visible portion of the room.
        lda     #$9B
        cmp.b   RECORD_X,x
        beq     .keep_swallowed_player_visible
        bcs     .keep_swallowed_player_visible
        sta.b   RECORD_X,x
.keep_swallowed_player_visible:
        lda     #$17
        cmp.b   RECORD_Y,x
        bcc     .done
        sta.b   RECORD_Y,x
.done:
        rts

; Roar/bite countdown starting values, paired by difficulty for Games 1, 2, and 3.
DragonBiteDelays:
        .byte   $D0,$E8                                          ; Game 1
        .byte   $F0,$F6                                          ; Game 2
        .byte   $F0,$F6                                          ; Game 3

; =============================================================================
; THE BAT, CASTLE GATES, AND MAGNET
;
; The bat chooses cargo, flies with it, and eventually looks for something else.
; Castle gates animate when touched by their keys and admit nearby objects.
; The magnet pulls eligible objects toward itself rather than moving itself.
; =============================================================================


; Animate the black bat. Its flight timer postpones choosing replacement cargo.
UpdateBat subroutine
        inc.b   bat_animation
        lda.b   bat_animation
        cmp     #$08
        bne     .check_flight_timer
        lda     #$00
        sta.b   bat_animation

; While committed to its current flight, keep moving along the stored direction.
.check_flight_timer:
        lda.b   bat_flight_timer
        beq     .choose_new_object
        inc.b   bat_flight_timer
        lda.b   bat_direction
        ldx     #bat_room
        ldy     #$03
        jsr     MoveObjectThroughWorld
        jmp     .move_bats_cargo

; Otherwise choose among the bat priorities, excluding its current cargo.
.choose_new_object:
        lda     #bat_room
        sta.b   creature_record
        lda     #$03
        sta.b   creature_speed
        lda     #<BatReactions
        sta.b   reaction_list
        lda     #>BatReactions
        sta.b   reaction_list+1
        lda.b   bat_carried_record
        sta.b   ignored_record
        jsr     MoveCreature

; Take the candidate only when it shares the bat room and is nearby.
        ldy.b   reaction_index
        lda     (reaction_list),y
        beq     .move_bats_cargo
        iny
        lda     (reaction_list),y
        tax
        lda.b   RECORD_ROOM,x
        cmp.b   bat_room
        bne     .move_bats_cargo

; Check a small neighborhood on both axes before swapping cargo.
        lda.b   RECORD_X,x
        sec
        sbc.b   bat_x
        clc
        adc     #$04
        and     #$F8
        bne     .move_bats_cargo
        lda.b   RECORD_Y,x
        sec
        sbc.b   bat_y
        clc
        adc     #$04
        and     #$F8
        bne     .move_bats_cargo

; Carry the new record and restart the flight timer.
        stx.b   bat_carried_record
        lda     #$10
        sta.b   bat_flight_timer

; The carried object follows beside the bat, including across room boundaries.
.move_bats_cargo:
        ldx.b   bat_carried_record
        lda.b   bat_room
        sta.b   RECORD_ROOM,x
        lda.b   bat_x
        clc
        adc     #$08
        sta.b   RECORD_X,x
        lda.b   bat_y
        sta.b   RECORD_Y,x

; If the bat takes the player held object, release the player attachment.
        lda.b   bat_carried_record
        ldy.b   held_object
        cmp.w   ObjectCatalog,y
        bne     .done
        lda     #OBJ_NONE
        sta.b   held_object
.done:
        rts

; Bat cargo priorities: chalice, sword, bridge, keys, dragons, then magnet.
BatReactions:
        .byte   bat_room,chalice_room
        .byte   bat_room,sword_room
        .byte   bat_room,bridge_room
        .byte   bat_room,gold_key_room
        .byte   bat_room,white_key_room
        .byte   bat_room,black_key_room
        .byte   bat_room,rhindle_room
        .byte   bat_room,yorgle_room
        .byte   bat_room,grundle_room
        .byte   bat_room,magnet_room
        .byte   $00                                              ; End of priorities

; Update the Golden, White, and Black Castle gates in turn.
UpdateCastleGates subroutine
        ldy     #$02

; The matching colored key starts a gate animation when it touches the gate.
.next_gate:
        ldx.w   GateObjects,y
        jsr     FindTouchingObject
        sta.b   contact_object
        cmp.w   GateKeys,y
        bne     .check_open_gate
        tya
        tax
        inc.b   gold_gate_state,x

; A fully closed gate admits nothing. An open one checks the player first.
.check_open_gate:
        tya
        tax
        lda.b   gold_gate_state,x
        cmp     #$1C
        beq     .animate_gate
        lda.w   GateObjects,y
        jsr     IsTouchingPlayer
        beq     .check_object_entry

; Player contact enters the castle and holds its gate open.
        lda     #$01
        sta.b   gold_gate_state,x
        ldx     #player_room
        jmp     .enter_castle

; Other displayed objects can also enter an open castle gate.
.check_object_entry:
        lda.b   contact_object
        cmp     #OBJ_NONE
        beq     .no_object_entry
        ldx.b   contact_object
        sty.b   saved_index
        jsr     FindObjectPosition
        ldy.b   saved_index
        ldx.b   record_pointer
        jmp     .enter_castle
.no_object_entry:
        jmp     .animate_gate

; Use this gate interior room and place the entering record near its bottom.
.enter_castle:
        lda.w   CastleInteriorRooms,y
        sta.b   RECORD_ROOM,x
        lda     #$10
        sta.b   RECORD_Y,x

; Open and closed states hold. Intermediate states advance through opening
; or closing frames, wrapping back to the open state.
.animate_gate:
        tya
        tax
        lda.b   gold_gate_state,x
        cmp     #$01
        beq     .advance_gate
        cmp     #$1C
        beq     .advance_gate
        inc.b   gold_gate_state,x
        lda.b   gold_gate_state,x
        cmp     #$38
        bne     .advance_gate
        lda     #$01
        sta.b   gold_gate_state,x
.advance_gate:
        dey
        bmi     .done
        jmp     .next_gate
.done:
        rts

; Golden, White, and Black Castle gates.
GateObjects:
        .byte   OBJ_GOLD_GATE,OBJ_WHITE_GATE,OBJ_BLACK_GATE

; The corresponding key for each gate.
GateKeys:
        .byte   OBJ_GOLD_KEY,OBJ_WHITE_KEY,OBJ_BLACK_KEY

; The rooms reached by entering the gates.
CastleInteriorRooms:
        .byte   ROOM_GOLD_TREASURY,ROOM_WHITE_CASTLE_INTERIOR,ROOM_BLACK_CASTLE_INTERIOR

; The castle facades reached when leaving their interiors.
CastleOutsideRooms:
        .byte   ROOM_GOLD_CASTLE,ROOM_WHITE_CASTLE,ROOM_BLACK_CASTLE

; Attract one eligible item toward the magnet. The search order prefers keys,
; then sword, bridge, and chalice. Creatures and the dot are unaffected.
UpdateMagnet subroutine
        lda.b   magnet_y
        sec
        sbc     #$08
        sta.b   magnet_y
        lda     #$00
        sta.b   ignored_record

; Aim slightly above the magnet so attracted items meet its useful pickup area.
        lda     #<MagnetReactions
        sta.b   reaction_list
        lda     #>MagnetReactions
        sta.b   reaction_list+1
        jsr     ChooseReaction

; The chosen pair leaves X pointing at the attracted item; move it one step.
        lda.b   move_direction
        beq     .restore_magnet_position
        ldy     #$01
        jsr     MoveObjectThroughWorld

; Restore the magnet coordinate used only as the attraction target.
.restore_magnet_position:
        lda.b   magnet_y
        clc
        adc     #$08
        sta.b   magnet_y
        rts

; Magnet attraction priorities: Gold Key, White Key, Black Key, sword, bridge, chalice.
MagnetReactions:
        .byte   gold_key_room,magnet_room
        .byte   white_key_room,magnet_room
        .byte   black_key_room,magnet_room
        .byte   sword_room,magnet_room
        .byte   bridge_room,magnet_room
        .byte   chalice_room,magnet_room
        .byte   $00                                              ; End of priorities

; Dark mazes use a movable light patch centered around the player.
; In other rooms its vertical position disables the patch.
UpdateDarkRoomLight subroutine
        lda.b   player_room
        jsr     FindRoomRecord
        ldy     #$02
        lda     (record_pointer),y
        cmp     #$08
        beq     .follow_player
        lda     #$00
        sta.b   light_y
        jmp     .done

; Follow the player with a rectangular region that reveals nearby walls.
.follow_player:
        lda.b   player_room
        sta.b   light_room
        lda.b   player_x
        sec
        sbc     #$0E
        sta.b   light_x
        lda.b   player_y
        clc
        adc     #$0E
        sta.b   light_y

; Clamp the region horizontally so it remains visible near screen edges.
        lda.b   light_x
        cmp     #$F0
        bcc     .clamp_right
        lda     #$01
        sta.b   light_x
        jmp     .done
.clamp_right:
        cmp     #$82
        bcc     .done
        lda     #$81
        sta.b   light_x
.done:
        rts

; =============================================================================
; SOUNDS AND THE VICTORY DISPLAY
;
; Events select a sound and a remaining duration. The turn loop advances it.
; Returning the chalice to the Golden Castle also cycles the room color.
; =============================================================================


; Silence expired effects; otherwise advance the event sound once per turn.
PlayGameSound subroutine
        lda.b   sound_remaining
        bne     .dispatch_effect
        sta.b   AUDV0
        sta.b   AUDV1
        rts

; Dispatch victory, roar, swallow, dragon death, drop, or pickup.
.dispatch_effect:
        dec.b   sound_remaining
        lda.b   sound_effect
        beq     .victory
        cmp     #SOUND_ROAR
        beq     .dragon_roar
        cmp     #SOUND_SWALLOW
        beq     .swallowed
        cmp     #SOUND_DRAGON_SLAIN
        beq     .dragon_slain
        cmp     #SOUND_DROP
        beq     .drop_object
        cmp     #SOUND_PICKUP
        beq     .pick_up_object
        rts

; Victory also changes the wall color as the celebration progresses.
.victory:
        lda.b   sound_remaining
        sta.b   COLUPF
        sta.b   AUDC0
        lsr
        sta.b   AUDV0
        lsr
        lsr
        sta.b   AUDF0
        rts

; Alternate tones during a dragon roar.
.dragon_roar:
        lda.b   sound_remaining
        lsr
        lda     #$03
        bcs     .roar_tone
        lda     #$08
.roar_tone:
        sta.b   AUDC0
        lda.b   sound_remaining
        sta.b   AUDV0
        lsr
        lsr
        clc
        adc     #$1C
        sta.b   AUDF0
        rts

; The swallow effect changes pitch as it fades.
.swallowed:
        lda     #$06
        sta.b   AUDC0
        lda.b   sound_remaining
        eor     #$0F
        sta.b   AUDF0
        lda.b   sound_remaining
        lsr
        clc
        adc     #$08
        sta.b   AUDV0
        rts

; A slain dragon has its own short sound.
.dragon_slain:
        lda     #$04
        sta.b   AUDC0
        lda.b   sound_remaining
        sta.b   AUDV0
        eor     #$1F
        sta.b   AUDF0
        rts

; Drop and pickup share their final tone/volume setup.
.drop_object:
        lda.b   sound_remaining
        eor     #$03
.item_sound:
        sta.b   AUDF0
        lda     #$05
        sta.b   AUDV0
        lda     #$06
        sta.b   AUDC0
        rts

; Pickup starts that same item sound from its own pitch.
.pick_up_object:
        lda.b   sound_remaining
        jmp     .item_sound

; =============================================================================
; ROOM WALLS AND OBJECT PICTURES
;
; Sprite bytes below are successive rows, with the leftmost pixel at bit 7.
; One stored row is drawn twice vertically. A zero ends the visible shape.
; Wall rows show a complete room width: the three stored bytes describe one
; half, which the room either mirrors or repeats. Thin side barriers are separate.
; =============================================================================


; Walls used by: Secret Passage.
WallsSecretPassage:
; The bottom wall band is shared with the first band of the next layout.
        .byte   $F0,$FF,$FF                                      ; |████████████████████████████████████████|
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $00,$00,$00                                      ; |                                        |
; Shared bottom band:                                              |████████████████        ████████████████|

; Walls used by: West Hall, Below Gold Castle, Robinett Credits.
WallsOpenHall:
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $F0,$FF,$FF                                      ; |████████████████████████████████████████|

; Walls used by: Side Passage 0C, Side Passage 0D.
WallsSidePassage:
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|

; Walls used by: Game Selection, Gold Treasury, Purple Chamber.
WallsEnclosedRoom:
        .byte   $F0,$FF,$FF                                      ; |████████████████████████████████████████|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|

; Choose the first state ceiling that includes the current state.
GateImages:
        .byte   $04                                              ; Inclusive state ceiling
        .word   GateImage+12
        .byte   $08                                              ; Inclusive state ceiling
        .word   GateImage+10
        .byte   $0C                                              ; Inclusive state ceiling
        .word   GateImage+8
        .byte   $10                                              ; Inclusive state ceiling
        .word   GateImage+6
        .byte   $14                                              ; Inclusive state ceiling
        .word   GateImage+4
        .byte   $18                                              ; Inclusive state ceiling
        .word   GateImage+2
        .byte   $1C                                              ; Inclusive state ceiling
        .word   GateImage
        .byte   $20                                              ; Inclusive state ceiling
        .word   GateImage+2
        .byte   $24                                              ; Inclusive state ceiling
        .word   GateImage+4
        .byte   $28                                              ; Inclusive state ceiling
        .word   GateImage+6
        .byte   $2C                                              ; Inclusive state ceiling
        .word   GateImage+8
        .byte   $30                                              ; Inclusive state ceiling
        .word   GateImage+10
        .byte   $FF                                              ; Inclusive state ceiling
        .word   GateImage+12

; Castle gate: opening and closing select shorter suffixes of this shape.
GateImage:
        .byte   $FE                                              ; |███████ |
        .byte   $AA                                              ; |█ █ █ █ |
        .byte   $FE                                              ; |███████ |
        .byte   $AA                                              ; |█ █ █ █ |
        .byte   $FE                                              ; |███████ |
        .byte   $AA                                              ; |█ █ █ █ |
        .byte   $FE                                              ; |███████ |
        .byte   $AA                                              ; |█ █ █ █ |
        .byte   $FE                                              ; |███████ |
        .byte   $AA                                              ; |█ █ █ █ |
        .byte   $FE                                              ; |███████ |
        .byte   $AA                                              ; |█ █ █ █ |
        .byte   $FE                                              ; |███████ |
        .byte   $AA                                              ; |█ █ █ █ |
        .byte   $FE                                              ; |███████ |
        .byte   $AA                                              ; |█ █ █ █ |
        .byte   $00                                              ; |        |  End of shape

; Walls used by: Black Castle Interior.
WallsBlackCastleInterior:
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|

; Walls used by: Blue Maze North.
WallsBlueMazeNorth:
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|
        .byte   $00,$0C,$0C                                      ; |        ██    ██        ██    ██        |
        .byte   $F0,$0C,$3C                                      ; |████    ██    ████    ████    ██    ████|
        .byte   $F0,$0C,$00                                      ; |████    ██                    ██    ████|
        .byte   $F0,$FF,$3F                                      ; |██████████████████    ██████████████████|
        .byte   $00,$30,$30                                      ; |      ██        ██    ██        ██      |
        .byte   $F0,$33,$3F                                      ; |████  ██  ████████    ████████  ██  ████|

; Walls used by: Blue Maze 05.
WallsBlueMaze05:
        .byte   $F0,$FF,$FF                                      ; |████████████████████████████████████████|
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $F0,$FC,$FF                                      ; |██████████  ████████████████  ██████████|
        .byte   $F0,$00,$C0                                      ; |████              ████              ████|
        .byte   $F0,$3F,$CF                                      ; |████  ██████████  ████  ██████████  ████|
        .byte   $00,$30,$CC                                      ; |      ██      ██  ████  ██      ██      |
        .byte   $F0,$F3,$CC                                      ; |████████  ██  ██  ████  ██  ██  ████████|

; Walls used by: Blue Maze South.
WallsBlueMazeSouth:
        .byte   $F0,$F3,$0C                                      ; |████████  ██  ██        ██  ██  ████████|
        .byte   $00,$30,$0C                                      ; |      ██      ██        ██      ██      |
        .byte   $F0,$3F,$0F                                      ; |████  ██████████        ██████████  ████|
        .byte   $F0,$00,$00                                      ; |████                                ████|
        .byte   $F0,$F0,$00                                      ; |████████                        ████████|
        .byte   $00,$30,$00                                      ; |      ██                        ██      |
        .byte   $F0,$FF,$FF                                      ; |████████████████████████████████████████|

; Walls used by: Blue Maze Center.
WallsBlueMazeCenter:
        .byte   $F0,$33,$3F                                      ; |████  ██  ████████    ████████  ██  ████|
        .byte   $00,$30,$3C                                      ; |      ██      ████    ████      ██      |
        .byte   $F0,$FF,$3C                                      ; |████████████  ████    ████  ████████████|
        .byte   $00,$03,$3C                                      ; |          ██  ████    ████  ██          |
        .byte   $F0,$33,$3C                                      ; |████  ██  ██  ████    ████  ██  ██  ████|
        .byte   $00,$33,$0C                                      ; |      ██  ██  ██        ██  ██  ██      |
        .byte   $F0,$F3,$0C                                      ; |████████  ██  ██        ██  ██  ████████|

; Walls used by: Blue Maze Entrance.
WallsBlueMazeEntrance:
        .byte   $F0,$F3,$CC                                      ; |████████  ██  ██  ████  ██  ██  ████████|
        .byte   $00,$33,$0C                                      ; |      ██  ██  ██        ██  ██  ██      |
        .byte   $F0,$33,$FC                                      ; |████  ██  ██  ████████████  ██  ██  ████|
        .byte   $00,$33,$00                                      ; |      ██  ██                ██  ██      |
        .byte   $F0,$F3,$FF                                      ; |████████  ████████████████████  ████████|
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|

; Walls used by: Dark Catacombs Center.
WallsCatacombsCenter:
; The bottom wall band is shared with the first band of the next layout.
        .byte   $F0,$FF,$CC                                      ; |████████████  ██  ████  ██  ████████████|
        .byte   $00,$00,$CC                                      ; |              ██  ████  ██              |
        .byte   $F0,$03,$CF                                      ; |████      ██████  ████  ██████      ████|
        .byte   $00,$03,$00                                      ; |          ██                ██          |
        .byte   $F0,$F3,$FC                                      ; |████████  ██  ████████████  ██  ████████|
        .byte   $00,$33,$0C                                      ; |      ██  ██  ██        ██  ██  ██      |
; Shared bottom band:                                              |████  ██  ██  ██  ████  ██  ██  ██  ████|

; Walls used by: Dark Catacombs Side.
WallsCatacombsSide:
        .byte   $F0,$33,$CC                                      ; |████  ██  ██  ██  ████  ██  ██  ██  ████|
        .byte   $00,$30,$CC                                      ; |      ██      ██  ████  ██      ██      |
        .byte   $00,$3F,$CF                                      ; |      ██████████  ████  ██████████      |
        .byte   $00,$00,$C0                                      ; |                  ████                  |
        .byte   $00,$3F,$C3                                      ; |      ████████    ████    ████████      |
        .byte   $00,$30,$C0                                      ; |      ██          ████          ██      |
        .byte   $F0,$FF,$FF                                      ; |████████████████████████████████████████|

; Walls used by: Dark Catacombs Entrance.
WallsCatacombsEntrance:
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|
        .byte   $00,$30,$00                                      ; |      ██                        ██      |
        .byte   $F0,$30,$FF                                      ; |████  ██    ████████████████    ██  ████|
        .byte   $00,$30,$C0                                      ; |      ██          ████          ██      |
        .byte   $F0,$F3,$C0                                      ; |████████  ██      ████      ██  ████████|
        .byte   $00,$03,$C0                                      ; |          ██      ████      ██          |
        .byte   $F0,$FF,$CC                                      ; |████████████  ██  ████  ██  ████████████|

; Walls used by: White Castle, Black Castle, Gold Castle.
WallsCastle:
        .byte   $F0,$FE,$15                                      ; |███████████ █ █ █      █ █ █ ███████████|
        .byte   $30,$03,$1F                                      ; |██        ███████      ███████        ██|
        .byte   $30,$03,$FF                                      ; |██        ████████████████████        ██|
        .byte   $30,$00,$FF                                      ; |██          ████████████████          ██|
        .byte   $30,$00,$3F                                      ; |██          ██████    ██████          ██|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|

; Golden Castle gate position.
GoldGatePosition:
        .byte   ROOM_GOLD_CASTLE,$4D,$31                         ; Room, x, y

; White Castle gate position.
WhiteGatePosition:
        .byte   ROOM_WHITE_CASTLE,$4D,$31                        ; Room, x, y

; Black Castle gate position.
BlackGatePosition:
        .byte   ROOM_BLACK_CASTLE,$4D,$31                        ; Room, x, y

; This object has a fixed state; its position can still move.
LightState:
        .byte   $00

; Choose the first state ceiling that includes the current state.
LightImages:
        .byte   $FF                                              ; Inclusive state ceiling
        .word   LightImage

; Dark-room light patch: a solid enlarged rectangle used to reveal nearby walls.
LightImage:
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $FF                                              ; |████████|
        .byte   $00                                              ; |        |  End of shape

; Walls used by: Red Maze 17.
WallsRedMaze17:
; The bottom wall band is shared with the first band of the next layout.
        .byte   $F0,$FF,$FF                                      ; |████████████████████████████████████████|
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|
        .byte   $00,$00,$0C                                      ; |              ██        ██              |
        .byte   $F0,$FF,$0C                                      ; |████████████  ██        ██  ████████████|
        .byte   $F0,$03,$CC                                      ; |████      ██  ██  ████  ██  ██      ████|
; Shared bottom band:                                              |████  ██  ██████  ████  ██████  ██  ████|

; Walls used by: Red Maze South.
WallsRedMazeSouth:
        .byte   $F0,$33,$CF                                      ; |████  ██  ██████  ████  ██████  ██  ████|
        .byte   $F0,$30,$00                                      ; |████  ██                        ██  ████|
        .byte   $F0,$33,$FF                                      ; |████  ██  ████████████████████  ██  ████|
        .byte   $00,$33,$00                                      ; |      ██  ██                ██  ██      |
        .byte   $F0,$FF,$00                                      ; |████████████                ████████████|
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|

; Walls used by: Red Maze North.
WallsRedMazeNorth:
; The bottom wall band is shared with the first band of the next layout.
        .byte   $F0,$FF,$FF                                      ; |████████████████████████████████████████|
        .byte   $00,$00,$C0                                      ; |                  ████                  |
        .byte   $F0,$FF,$CF                                      ; |████████████████  ████  ████████████████|
        .byte   $00,$00,$CC                                      ; |              ██  ████  ██              |
        .byte   $F0,$33,$FF                                      ; |████  ██  ████████████████████  ██  ████|
        .byte   $F0,$33,$00                                      ; |████  ██  ██                ██  ██  ████|
; Shared bottom band:                                              |████  ██████  ██        ██  ██████  ████|

; Walls used by: White Castle Interior.
WallsWhiteCastleInterior:
        .byte   $F0,$3F,$0C                                      ; |████  ██████  ██        ██  ██████  ████|
        .byte   $F0,$00,$0C                                      ; |████          ██        ██          ████|
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|
        .byte   $00,$30,$00                                      ; |      ██                        ██      |
        .byte   $F0,$30,$00                                      ; |████  ██                        ██  ████|
        .byte   $00,$30,$00                                      ; |      ██                        ██      |
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|

; Walls used by: Above White Castle, Red Hall.
WallsTopOpening:
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $30,$00,$00                                      ; |██                                    ██|
        .byte   $F0,$FF,$FF                                      ; |████████████████████████████████████████|

; Walls used by: Black Maze 13.
WallsBlackMaze13:
; The bottom wall band is shared with the first band of the next layout.
        .byte   $F0,$F0,$FF                                      ; |████████    ████████████████    ████████|
        .byte   $00,$00,$03                                      ; |            ██            ██            |
        .byte   $F0,$FF,$03                                      ; |██████████████            ██████████████|
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $30,$3F,$FF                                      ; |██    ████████████████████████████    ██|
        .byte   $00,$30,$00                                      ; |      ██                        ██      |
; Shared bottom band:                                              |████████    ████████████████    ████████|

; Walls used by: Black Maze Dot.
WallsBlackMazeDot:
        .byte   $F0,$F0,$FF                                      ; |████████    ████████████████    ████████|
        .byte   $30,$00,$00                                      ; |██                  ██                  |
        .byte   $30,$3F,$FF                                      ; |██    ████████████████    ██████████████|
        .byte   $00,$30,$00                                      ; |      ██                  ██            |
        .byte   $F0,$F0,$FF                                      ; |████████    ████████████████    ████████|
        .byte   $30,$00,$03                                      ; |██          ██      ██          ██      |
        .byte   $F0,$F0,$FF                                      ; |████████    ████████████████    ████████|

; Walls used by: Black Maze 14.
WallsBlackMaze14:
; The bottom wall band is shared with the first band of the next layout.
        .byte   $F0,$FF,$FF                                      ; |████████████████████████████████████████|
        .byte   $00,$00,$C0                                      ; |                  ██                  ██|
        .byte   $F0,$FF,$CF                                      ; |████████████████  ██████████████████  ██|
        .byte   $00,$00,$0C                                      ; |              ██                  ██    |
        .byte   $F0,$0F,$FF                                      ; |████    ████████████████    ████████████|
        .byte   $00,$0F,$C0                                      ; |        ████      ██        ████      ██|
; Shared bottom band:                                              |██  ██  ████  ██  ████  ██  ████  ██  ██|

; Walls used by: Black Maze Entrance.
WallsBlackMazeEntrance:
        .byte   $30,$CF,$CC                                      ; |██  ██  ████  ██  ████  ██  ████  ██  ██|
        .byte   $00,$C0,$CC                                      ; |    ██        ██  ████  ██        ██    |
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|
        .byte   $00,$00,$00                                      ; |                                        |
        .byte   $F0,$FF,$0F                                      ; |████████████████        ████████████████|

; This object has a fixed state; its position can still move.
BridgeState:
        .byte   $00

; Choose the first state ceiling that includes the current state.
BridgeImages:
        .byte   $FF                                              ; Inclusive state ceiling
        .word   BridgeImage

; Bridge: enlarged horizontally when displayed.
BridgeImage:
        .byte   $C3                                              ; |██    ██|
        .byte   $C3                                              ; |██    ██|
        .byte   $C3                                              ; |██    ██|
        .byte   $C3                                              ; |██    ██|
        .byte   $42                                              ; | █    █ |
        .byte   $42                                              ; | █    █ |
        .byte   $42                                              ; | █    █ |
        .byte   $42                                              ; | █    █ |
        .byte   $42                                              ; | █    █ |
        .byte   $42                                              ; | █    █ |
        .byte   $42                                              ; | █    █ |
        .byte   $42                                              ; | █    █ |
        .byte   $42                                              ; | █    █ |
        .byte   $42                                              ; | █    █ |
        .byte   $42                                              ; | █    █ |
        .byte   $42                                              ; | █    █ |
        .byte   $42                                              ; | █    █ |
        .byte   $42                                              ; | █    █ |
        .byte   $42                                              ; | █    █ |
        .byte   $42                                              ; | █    █ |
        .byte   $C3                                              ; |██    ██|
        .byte   $C3                                              ; |██    ██|
        .byte   $C3                                              ; |██    ██|
        .byte   $C3                                              ; |██    ██|
        .byte   $00                                              ; |        |  End of shape

; Game 1 numeral.
GameOneImage:
        .byte   $04                                              ; |     █  |
        .byte   $0C                                              ; |    ██  |
        .byte   $04                                              ; |     █  |
        .byte   $04                                              ; |     █  |
        .byte   $04                                              ; |     █  |
        .byte   $04                                              ; |     █  |
        .byte   $0E                                              ; |    ███ |
        .byte   $00                                              ; |        |  End of shape

; This object has a fixed state; its position can still move.
KeyState:
        .byte   $00

; Choose the first state ceiling that includes the current state.
KeyImages:
        .byte   $FF                                              ; Inclusive state ceiling
        .word   KeyImage

; Key shape shared by the Gold, White, and Black Keys.
KeyImage:
        .byte   $07                                              ; |     ███|
        .byte   $FD                                              ; |██████ █|
        .byte   $A7                                              ; |█ █  ███|
        .byte   $00                                              ; |        |  End of shape

; Game 2 numeral.
GameTwoImage:
        .byte   $0E                                              ; |    ███ |
        .byte   $11                                              ; |   █   █|
        .byte   $01                                              ; |       █|
        .byte   $02                                              ; |      █ |
        .byte   $04                                              ; |     █  |
        .byte   $08                                              ; |    █   |
        .byte   $1F                                              ; |   █████|
        .byte   $00                                              ; |        |  End of shape

; Game 3 numeral.
GameThreeImage:
        .byte   $0E                                              ; |    ███ |
        .byte   $11                                              ; |   █   █|
        .byte   $01                                              ; |       █|
        .byte   $06                                              ; |     ██ |
        .byte   $01                                              ; |       █|
        .byte   $11                                              ; |   █   █|
        .byte   $0E                                              ; |    ███ |
        .byte   $00                                              ; |        |  End of shape

; Choose the first state ceiling that includes the current state.
BatImages:
        .byte   $03                                              ; Inclusive state ceiling
        .word   BatFrameOne
        .byte   $FF                                              ; Inclusive state ceiling
        .word   BatFrameTwo

; Black bat, animation frame one.
BatFrameOne:
        .byte   $81                                              ; |█      █|
        .byte   $81                                              ; |█      █|
        .byte   $C3                                              ; |██    ██|
        .byte   $C3                                              ; |██    ██|
        .byte   $FF                                              ; |████████|
        .byte   $5A                                              ; | █ ██ █ |
        .byte   $66                                              ; | ██  ██ |
        .byte   $00                                              ; |        |  End of shape

; Black bat, animation frame two.
BatFrameTwo:
        .byte   $01                                              ; |       █|
        .byte   $80                                              ; |█       |
        .byte   $01                                              ; |       █|
        .byte   $80                                              ; |█       |
        .byte   $3C                                              ; |  ████  |
        .byte   $5A                                              ; | █ ██ █ |
        .byte   $66                                              ; | ██  ██ |
        .byte   $C3                                              ; |██    ██|
        .byte   $81                                              ; |█      █|
        .byte   $81                                              ; |█      █|
        .byte   $81                                              ; |█      █|
        .byte   $00                                              ; |        |  End of shape

; Choose the first state ceiling that includes the current state.
DragonImages:
        .byte   $00                                              ; Inclusive state ceiling
        .word   DragonWalkingImage
        .byte   $01                                              ; Inclusive state ceiling
        .word   DragonDeadImage
        .byte   $02                                              ; Inclusive state ceiling
        .word   DragonWalkingImage
        .byte   $FF                                              ; Inclusive state ceiling
        .word   DragonBitingImage

; Walking dragon, shared by Rhindle, Yorgle, and Grundle.
DragonWalkingImage:
        .byte   $06                                              ; |     ██ |
        .byte   $0F                                              ; |    ████|
        .byte   $F3                                              ; |████  ██|
        .byte   $FE                                              ; |███████ |
        .byte   $0E                                              ; |    ███ |
        .byte   $04                                              ; |     █  |
        .byte   $04                                              ; |     █  |
        .byte   $1E                                              ; |   ████ |
        .byte   $3F                                              ; |  ██████|
        .byte   $7F                                              ; | ███████|
        .byte   $E3                                              ; |███   ██|
        .byte   $C3                                              ; |██    ██|
        .byte   $C3                                              ; |██    ██|
        .byte   $C7                                              ; |██   ███|
        .byte   $FF                                              ; |████████|
        .byte   $3C                                              ; |  ████  |
        .byte   $08                                              ; |    █   |
        .byte   $8F                                              ; |█   ████|
        .byte   $E1                                              ; |███    █|
        .byte   $3F                                              ; |  ██████|
        .byte   $00                                              ; |        |  End of shape

; Roaring and biting dragon.
DragonBitingImage:
        .byte   $80                                              ; |█       |
        .byte   $40                                              ; | █      |
        .byte   $26                                              ; |  █  ██ |
        .byte   $1F                                              ; |   █████|
        .byte   $0B                                              ; |    █ ██|
        .byte   $0E                                              ; |    ███ |
        .byte   $1E                                              ; |   ████ |
        .byte   $24                                              ; |  █  █  |
        .byte   $44                                              ; | █   █  |
        .byte   $8E                                              ; |█   ███ |
        .byte   $1E                                              ; |   ████ |
        .byte   $3F                                              ; |  ██████|
        .byte   $7F                                              ; | ███████|
        .byte   $7F                                              ; | ███████|
        .byte   $7F                                              ; | ███████|
        .byte   $7F                                              ; | ███████|
        .byte   $3E                                              ; |  █████ |
        .byte   $1C                                              ; |   ███  |
        .byte   $08                                              ; |    █   |
        .byte   $F8                                              ; |█████   |
        .byte   $80                                              ; |█       |
        .byte   $E0                                              ; |███     |
        .byte   $00                                              ; |        |  End of shape

; Slain dragon.
DragonDeadImage:
        .byte   $0C                                              ; |    ██  |
        .byte   $0C                                              ; |    ██  |
        .byte   $0C                                              ; |    ██  |
        .byte   $0E                                              ; |    ███ |
        .byte   $1B                                              ; |   ██ ██|
        .byte   $7F                                              ; | ███████|
        .byte   $CE                                              ; |██  ███ |
        .byte   $80                                              ; |█       |
        .byte   $FC                                              ; |██████  |
        .byte   $FE                                              ; |███████ |
        .byte   $FE                                              ; |███████ |
        .byte   $7E                                              ; | ██████ |
        .byte   $78                                              ; | ████   |
        .byte   $20                                              ; |  █     |
        .byte   $6E                                              ; | ██ ███ |
        .byte   $42                                              ; | █    █ |
        .byte   $7E                                              ; | ██████ |
        .byte   $00                                              ; |        |  End of shape

; This object has a fixed state; its position can still move.
SwordState:
        .byte   $00

; Choose the first state ceiling that includes the current state.
SwordImages:
        .byte   $FF                                              ; Inclusive state ceiling
        .word   SwordImage

; Sword.
SwordImage:
        .byte   $20                                              ; |  █     |
        .byte   $40                                              ; | █      |
        .byte   $FF                                              ; |████████|
        .byte   $40                                              ; | █      |
        .byte   $20                                              ; |  █     |
        .byte   $00                                              ; |        |  End of shape

; This object has a fixed state; its position can still move.
DotState:
        .byte   $00

; Choose the first state ceiling that includes the current state.
DotImages:
        .byte   $FF                                              ; Inclusive state ceiling
        .word   DotImage

; The Easter egg dot: one narrow pixel column.
DotImage:
        .byte   $80                                              ; |█       |
        .byte   $00                                              ; |        |  End of shape

; Warren Robinett credits: the lettering is stored as one tall bitmap.
RobinettCreditsImage:
        .byte   $F0                                              ; |████    |
        .byte   $80                                              ; |█       |
        .byte   $80                                              ; |█       |
        .byte   $80                                              ; |█       |
        .byte   $F4                                              ; |████ █  |
        .byte   $04                                              ; |     █  |
        .byte   $87                                              ; |█    ███|
        .byte   $E5                                              ; |███  █ █|
        .byte   $87                                              ; |█    ███|
        .byte   $80                                              ; |█       |
        .byte   $05                                              ; |     █ █|
        .byte   $E5                                              ; |███  █ █|
        .byte   $A7                                              ; |█ █  ███|
        .byte   $E1                                              ; |███    █|
        .byte   $87                                              ; |█    ███|
        .byte   $E0                                              ; |███     |
        .byte   $01                                              ; |       █|
        .byte   $E0                                              ; |███     |
        .byte   $A0                                              ; |█ █     |
        .byte   $F0                                              ; |████    |
        .byte   $01                                              ; |       █|
        .byte   $40                                              ; | █      |
        .byte   $E0                                              ; |███     |
        .byte   $40                                              ; | █      |
        .byte   $40                                              ; | █      |
        .byte   $40                                              ; | █      |
        .byte   $01                                              ; |       █|
        .byte   $E0                                              ; |███     |
        .byte   $A0                                              ; |█ █     |
        .byte   $E0                                              ; |███     |
        .byte   $80                                              ; |█       |
        .byte   $E0                                              ; |███     |
        .byte   $01                                              ; |       █|
        .byte   $20                                              ; |  █     |
        .byte   $20                                              ; |  █     |
        .byte   $E0                                              ; |███     |
        .byte   $A0                                              ; |█ █     |
        .byte   $E0                                              ; |███     |
        .byte   $01                                              ; |       █|
        .byte   $01                                              ; |       █|
        .byte   $01                                              ; |       █|
        .byte   $88                                              ; |█   █   |
        .byte   $A8                                              ; |█ █ █   |
        .byte   $A8                                              ; |█ █ █   |
        .byte   $A8                                              ; |█ █ █   |
        .byte   $F8                                              ; |█████   |
        .byte   $01                                              ; |       █|
        .byte   $E0                                              ; |███     |
        .byte   $A0                                              ; |█ █     |
        .byte   $F0                                              ; |████    |
        .byte   $01                                              ; |       █|
        .byte   $80                                              ; |█       |
        .byte   $E0                                              ; |███     |
        .byte   $8F                                              ; |█   ████|
        .byte   $89                                              ; |█   █  █|
        .byte   $0F                                              ; |    ████|
        .byte   $8A                                              ; |█   █ █ |
        .byte   $E9                                              ; |███ █  █|
        .byte   $80                                              ; |█       |
        .byte   $8E                                              ; |█   ███ |
        .byte   $0A                                              ; |    █ █ |
        .byte   $EE                                              ; |███ ███ |
        .byte   $A0                                              ; |█ █     |
        .byte   $E8                                              ; |███ █   |
        .byte   $88                                              ; |█   █   |
        .byte   $EE                                              ; |███ ███ |
        .byte   $0A                                              ; |    █ █ |
        .byte   $8E                                              ; |█   ███ |
        .byte   $E0                                              ; |███     |
        .byte   $A4                                              ; |█ █  █  |
        .byte   $A4                                              ; |█ █  █  |
        .byte   $04                                              ; |     █  |
        .byte   $80                                              ; |█       |
        .byte   $08                                              ; |    █   |
        .byte   $0E                                              ; |    ███ |
        .byte   $0A                                              ; |    █ █ |
        .byte   $0A                                              ; |    █ █ |
        .byte   $80                                              ; |█       |
        .byte   $0E                                              ; |    ███ |
        .byte   $0A                                              ; |    █ █ |
        .byte   $0E                                              ; |    ███ |
        .byte   $08                                              ; |    █   |
        .byte   $0E                                              ; |    ███ |
        .byte   $80                                              ; |█       |
        .byte   $04                                              ; |     █  |
        .byte   $0E                                              ; |    ███ |
        .byte   $04                                              ; |     █  |
        .byte   $04                                              ; |     █  |
        .byte   $04                                              ; |     █  |
        .byte   $80                                              ; |█       |
        .byte   $04                                              ; |     █  |
        .byte   $0E                                              ; |    ███ |
        .byte   $04                                              ; |     █  |
        .byte   $04                                              ; |     █  |
        .byte   $04                                              ; |     █  |
        .byte   $00                                              ; |        |  End of shape

; Warren Robinett credits position.
RobinettCreditsPosition:
        .byte   ROOM_ROBINETT_CREDITS,$50,$69                    ; Room, x, y

; This object has a fixed state; its position can still move.
RobinettCreditsState:
        .byte   $00

; Choose the first state ceiling that includes the current state.
RobinettCreditsImages:
        .byte   $FF                                              ; Inclusive state ceiling
        .word   RobinettCreditsImage

; This object has a fixed state; its position can still move.
ChaliceState:
        .byte   $00

; Choose the first state ceiling that includes the current state.
ChaliceImages:
        .byte   $FF                                              ; Inclusive state ceiling
        .word   ChaliceImage

; Enchanted chalice.
ChaliceImage:
        .byte   $81                                              ; |█      █|
        .byte   $81                                              ; |█      █|
        .byte   $C3                                              ; |██    ██|
        .byte   $7E                                              ; | ██████ |
        .byte   $7E                                              ; | ██████ |
        .byte   $3C                                              ; |  ████  |
        .byte   $18                                              ; |   ██   |
        .byte   $18                                              ; |   ██   |
        .byte   $7E                                              ; | ██████ |
        .byte   $00                                              ; |        |  End of shape

; This object has a fixed state; its position can still move.
EmptyObjectState:
        .byte   $00

; Choose the first state ceiling that includes the current state.
EmptyObjectImages:
        .byte   $FF                                              ; Inclusive state ceiling
        .word   EmptyObjectImage

; Blank shape for OBJ_NONE.
EmptyObjectImage:
        .byte   $00                                              ; |        |  End of shape

; Game-number position; the chosen game supplies its image.
GameNumberPosition:
        .byte   ROOM_GAME_SELECTION,$50,$40                      ; Room, x, y

; Choose the first state ceiling that includes the current state.
GameNumberImages:
        .byte   $01                                              ; Inclusive state ceiling
        .word   GameOneImage
        .byte   $03                                              ; Inclusive state ceiling
        .word   GameTwoImage
        .byte   $FF                                              ; Inclusive state ceiling
        .word   GameThreeImage

; This object has a fixed state; its position can still move.
MagnetState:
        .byte   $00

; Choose the first state ceiling that includes the current state.
MagnetImages:
        .byte   $FF                                              ; Inclusive state ceiling
        .word   MagnetImage

; Magnet.
MagnetImage:
        .byte   $3C                                              ; |  ████  |
        .byte   $7E                                              ; | ██████ |
        .byte   $E7                                              ; |███  ███|
        .byte   $C3                                              ; |██    ██|
        .byte   $C3                                              ; |██    ██|
        .byte   $C3                                              ; |██    ██|
        .byte   $C3                                              ; |██    ██|
        .byte   $C3                                              ; |██    ██|
        .byte   $00                                              ; |        |  End of shape

; =============================================================================
; THE KINGDOM MAP AND ITS INHABITANTS
;
; Room records bind wall pictures to colors and the four neighboring rooms.
; The castle facades are outside rooms; entering their gates changes rooms.
; Object records bind position, state, image choices, colors, and display size.
; =============================================================================


; Each record: wall picture; color/monochrome appearance; wall flags;
; then exits in the order up, right, down, left.
RoomTable:

; Room $00: Game Selection.
        .word   WallsEnclosedRoom
        .byte   $66,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_GAME_SELECTION,ROOM_GAME_SELECTION,ROOM_GAME_SELECTION,ROOM_GAME_SELECTION ; Up, right, down, left

; Room $01: West Hall.
        .word   WallsOpenHall
        .byte   $D8,$0A,$A1                                      ; Color, monochrome color, wall flags
        .byte   ROOM_BLUE_MAZE_ENTRANCE,ROOM_BELOW_GOLD_CASTLE,$80,ROOM_SECRET_PASSAGE ; Up, right, down, left

; Room $02: Below Gold Castle.
        .word   WallsOpenHall
        .byte   $C8,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_GOLD_CASTLE,ROOM_SECRET_PASSAGE,$83,ROOM_WEST_HALL ; Up, right, down, left

; Room $03: Secret Passage.
        .word   WallsSecretPassage
        .byte   $E8,$0A,$61                                      ; Color, monochrome color, wall flags
        .byte   ROOM_BLUE_MAZE_SOUTH,ROOM_WEST_HALL,$86,ROOM_BELOW_GOLD_CASTLE ; Up, right, down, left

; Room $04: Blue Maze North.
        .word   WallsBlueMazeNorth
        .byte   $86,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_BLACK_CASTLE,ROOM_BLUE_MAZE_05,ROOM_BLUE_MAZE_CENTER,ROOM_BLUE_MAZE_SOUTH ; Up, right, down, left

; Room $05: Blue Maze 05.
        .word   WallsBlueMaze05
        .byte   $86,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_RED_HALL,ROOM_BLUE_MAZE_SOUTH,ROOM_BLUE_MAZE_ENTRANCE,ROOM_BLUE_MAZE_NORTH ; Up, right, down, left

; Room $06: Blue Maze South.
        .word   WallsBlueMazeSouth
        .byte   $86,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_BLUE_MAZE_CENTER,ROOM_BLUE_MAZE_NORTH,ROOM_SECRET_PASSAGE,ROOM_BLUE_MAZE_05 ; Up, right, down, left

; Room $07: Blue Maze Center.
        .word   WallsBlueMazeCenter
        .byte   $86,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_BLUE_MAZE_NORTH,ROOM_BLUE_MAZE_ENTRANCE,ROOM_BLUE_MAZE_SOUTH,ROOM_BLUE_MAZE_ENTRANCE ; Up, right, down, left

; Room $08: Blue Maze Entrance.
        .word   WallsBlueMazeEntrance
        .byte   $86,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_BLUE_MAZE_05,ROOM_BLUE_MAZE_CENTER,ROOM_WEST_HALL,ROOM_BLUE_MAZE_CENTER ; Up, right, down, left

; Room $09: Dark Catacombs Center.
        .word   WallsCatacombsCenter
        .byte   $08,$08,$25                                      ; Color, monochrome color, wall flags
        .byte   ROOM_DARK_CATACOMBS_ENTRANCE,ROOM_DARK_CATACOMBS_ENTRANCE,ROOM_DARK_CATACOMBS_SIDE,ROOM_DARK_CATACOMBS_ENTRANCE ; Up, right, down, left

; Room $0A: Dark Catacombs Entrance.
        .word   WallsCatacombsEntrance
        .byte   $08,$08,$25                                      ; Color, monochrome color, wall flags
        .byte   ROOM_SECRET_PASSAGE,ROOM_DARK_CATACOMBS_CENTER,ROOM_DARK_CATACOMBS_CENTER,ROOM_DARK_CATACOMBS_CENTER ; Up, right, down, left

; Room $0B: Dark Catacombs Side.
        .word   WallsCatacombsSide
        .byte   $08,$08,$25                                      ; Color, monochrome color, wall flags
        .byte   ROOM_DARK_CATACOMBS_CENTER,ROOM_SIDE_PASSAGE_0C,ROOM_PURPLE_CHAMBER,ROOM_SIDE_PASSAGE_0D ; Up, right, down, left

; Room $0C: Side Passage 0C.
        .word   WallsSidePassage
        .byte   $98,$0A,$61                                      ; Color, monochrome color, wall flags
        .byte   ROOM_PURPLE_CHAMBER,ROOM_SIDE_PASSAGE_0D,ROOM_RED_HALL,ROOM_DARK_CATACOMBS_SIDE ; Up, right, down, left

; Room $0D: Side Passage 0D.
        .word   WallsSidePassage
        .byte   $B8,$0A,$A1                                      ; Color, monochrome color, wall flags
        .byte   ROOM_WHITE_CASTLE,ROOM_DARK_CATACOMBS_SIDE,ROOM_ABOVE_WHITE_CASTLE,ROOM_SIDE_PASSAGE_0C ; Up, right, down, left

; Room $0E: Above White Castle.
        .word   WallsTopOpening
        .byte   $A8,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_SIDE_PASSAGE_0D,ROOM_BLACK_CASTLE,ROOM_WHITE_CASTLE,ROOM_BLACK_CASTLE ; Up, right, down, left

; Room $0F: White Castle.
        .word   WallsCastle
        .byte   $0C,$0C,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_ABOVE_WHITE_CASTLE,ROOM_WHITE_CASTLE,ROOM_SIDE_PASSAGE_0D,ROOM_WHITE_CASTLE ; Up, right, down, left

; Room $10: Black Castle.
        .word   WallsCastle
        .byte   $00,$02,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_WEST_HALL,ROOM_PURPLE_CHAMBER,ROOM_BLUE_MAZE_NORTH,ROOM_PURPLE_CHAMBER ; Up, right, down, left

; Room $11: Gold Castle.
        .word   WallsCastle
        .byte   $1A,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_BLUE_MAZE_SOUTH,ROOM_SECRET_PASSAGE,ROOM_BELOW_GOLD_CASTLE,ROOM_WEST_HALL ; Up, right, down, left

; Room $12: Gold Treasury.
        .word   WallsEnclosedRoom
        .byte   $1A,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_GOLD_TREASURY,ROOM_GOLD_TREASURY,ROOM_GOLD_TREASURY,ROOM_GOLD_TREASURY ; Up, right, down, left

; Room $13: Black Maze 13.
        .word   WallsBlackMaze13
        .byte   $08,$08,$25                                      ; Color, monochrome color, wall flags
        .byte   ROOM_BLACK_MAZE_DOT,ROOM_BLACK_MAZE_14,ROOM_BLACK_MAZE_DOT,ROOM_BLACK_MAZE_ENTRANCE ; Up, right, down, left

; Room $14: Black Maze 14.
        .word   WallsBlackMaze14
        .byte   $08,$08,$24                                      ; Color, monochrome color, wall flags
        .byte   ROOM_BLACK_MAZE_ENTRANCE,ROOM_BLACK_MAZE_DOT,ROOM_BLACK_MAZE_ENTRANCE,ROOM_BLACK_MAZE_13 ; Up, right, down, left

; Room $15: Black Maze Dot.
        .word   WallsBlackMazeDot
        .byte   $08,$08,$24                                      ; Color, monochrome color, wall flags
        .byte   ROOM_BLACK_MAZE_13,ROOM_BLACK_MAZE_ENTRANCE,ROOM_BLACK_MAZE_13,ROOM_BLACK_MAZE_14 ; Up, right, down, left

; Room $16: Black Maze Entrance.
        .word   WallsBlackMazeEntrance
        .byte   $08,$08,$25                                      ; Color, monochrome color, wall flags
        .byte   ROOM_BLACK_MAZE_14,ROOM_BLACK_MAZE_13,ROOM_BLACK_CASTLE_INTERIOR,ROOM_BLACK_MAZE_DOT ; Up, right, down, left

; Room $17: Red Maze 17.
        .word   WallsRedMaze17
        .byte   $36,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_RED_MAZE_SOUTH,ROOM_RED_MAZE_NORTH,ROOM_RED_MAZE_SOUTH,ROOM_RED_MAZE_NORTH ; Up, right, down, left

; Room $18: Red Maze North.
        .word   WallsRedMazeNorth
        .byte   $36,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_WHITE_CASTLE_INTERIOR,ROOM_RED_MAZE_17,ROOM_WHITE_CASTLE_INTERIOR,ROOM_RED_MAZE_17 ; Up, right, down, left

; Room $19: Red Maze South.
        .word   WallsRedMazeSouth
        .byte   $36,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_RED_MAZE_17,ROOM_WHITE_CASTLE_INTERIOR,ROOM_RED_MAZE_17,ROOM_WHITE_CASTLE_INTERIOR ; Up, right, down, left

; Room $1A: White Castle Interior.
        .word   WallsWhiteCastleInterior
        .byte   $36,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_RED_MAZE_NORTH,ROOM_RED_MAZE_SOUTH,ROOM_RED_MAZE_NORTH,ROOM_RED_MAZE_SOUTH ; Up, right, down, left

; Room $1B: Black Castle Interior.
        .word   WallsBlackCastleInterior
        .byte   $36,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   $89,$89,$89,$89                                  ; Up, right, down, left

; Room $1C: Purple Chamber.
        .word   WallsEnclosedRoom
        .byte   $66,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_RED_HALL,ROOM_BLUE_MAZE_CENTER,$8C,ROOM_BLUE_MAZE_ENTRANCE ; Up, right, down, left

; Room $1D: Red Hall.
        .word   WallsTopOpening
        .byte   $36,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   $8F,ROOM_WEST_HALL,ROOM_BLACK_CASTLE,ROOM_SECRET_PASSAGE ; Up, right, down, left

; Room $1E: Robinett Credits.
        .word   WallsOpenHall
        .byte   $66,$0A,$21                                      ; Color, monochrome color, wall flags
        .byte   ROOM_BLUE_MAZE_SOUTH,ROOM_WEST_HALL,ROOM_BLUE_MAZE_SOUTH,ROOM_SECRET_PASSAGE ; Up, right, down, left

; Tagged exits select one destination for each of Games 1, 2, and 3.
GameDependentExits:
        .byte   ROOM_BLACK_CASTLE,ROOM_WHITE_CASTLE,ROOM_WHITE_CASTLE ; down from the west hall
        .byte   ROOM_BLUE_MAZE_05,ROOM_GOLD_CASTLE,ROOM_GOLD_CASTLE ; down from below the Golden Castle
        .byte   ROOM_RED_HALL,ROOM_DARK_CATACOMBS_ENTRANCE,ROOM_DARK_CATACOMBS_ENTRANCE ; down from the secret passage
        .byte   ROOM_PURPLE_CHAMBER,ROOM_BLACK_MAZE_ENTRANCE,ROOM_BLACK_MAZE_ENTRANCE ; edges of the Black Castle interior
        .byte   ROOM_BLACK_CASTLE_INTERIOR,ROOM_SIDE_PASSAGE_0C,ROOM_SIDE_PASSAGE_0C ; down from the purple chamber
        .byte   ROOM_SECRET_PASSAGE,ROOM_SIDE_PASSAGE_0C,ROOM_SIDE_PASSAGE_0C ; up from the red hall

; The three pointer fields name position, current state, and image choices.
; Their final bytes supply color, monochrome color, and display size.
ObjectCatalog:

; Dark Room Light.
        .word   light_room
        .word   LightState
        .word   LightImages
        .byte   $28,$0C,$07                                      ; Color, monochrome color, display size

; Gold Gate.
        .word   GoldGatePosition
        .word   gold_gate_state
        .word   GateImages
        .byte   $00,$00,$00                                      ; Color, monochrome color, display size

; White Gate.
        .word   WhiteGatePosition
        .word   white_gate_state
        .word   GateImages
        .byte   $00,$00,$00                                      ; Color, monochrome color, display size

; Black Gate.
        .word   BlackGatePosition
        .word   black_gate_state
        .word   GateImages
        .byte   $00,$00,$00                                      ; Color, monochrome color, display size

; Robinett Credits.
        .word   RobinettCreditsPosition
        .word   RobinettCreditsState
        .word   RobinettCreditsImages
        .byte   $CB,$00,$00                                      ; Color, monochrome color, display size

; Game Number.
        .word   GameNumberPosition
        .word   game_selection
        .word   GameNumberImages
        .byte   $C8,$00,$00                                      ; Color, monochrome color, display size

; Rhindle.
        .word   rhindle_room
        .word   rhindle_state
        .word   DragonImages
        .byte   $36,$0E,$00                                      ; Color, monochrome color, display size

; Yorgle.
        .word   yorgle_room
        .word   yorgle_state
        .word   DragonImages
        .byte   $1A,$06,$00                                      ; Color, monochrome color, display size

; Grundle.
        .word   grundle_room
        .word   grundle_state
        .word   DragonImages
        .byte   $C8,$00,$00                                      ; Color, monochrome color, display size

; Sword.
        .word   sword_room
        .word   SwordState
        .word   SwordImages
        .byte   $1A,$06,$00                                      ; Color, monochrome color, display size

; Bridge.
        .word   bridge_room
        .word   BridgeState
        .word   BridgeImages
        .byte   $66,$02,$07                                      ; Color, monochrome color, display size

; Gold Key.
        .word   gold_key_room
        .word   KeyState
        .word   KeyImages
        .byte   $1A,$06,$00                                      ; Color, monochrome color, display size

; White Key.
        .word   white_key_room
        .word   KeyState
        .word   KeyImages
        .byte   $0E,$0E,$00                                      ; Color, monochrome color, display size

; Black Key.
        .word   black_key_room
        .word   KeyState
        .word   KeyImages
        .byte   $00,$00,$00                                      ; Color, monochrome color, display size

; Bat.
        .word   bat_room
        .word   bat_animation
        .word   BatImages
        .byte   $00,$00,$00                                      ; Color, monochrome color, display size

; Easter Egg Dot.
        .word   dot_room
        .word   DotState
        .word   DotImages
        .byte   $08,$08,$00                                      ; Color, monochrome color, display size

; Chalice.
        .word   chalice_room
        .word   ChaliceState
        .word   ChaliceImages
        .byte   $CB,$06,$00                                      ; Color, monochrome color, display size

; Magnet.
        .word   magnet_room
        .word   MagnetState
        .word   MagnetImages
        .byte   $00,$06,$00                                      ; Color, monochrome color, display size

; None.
        .word   bridge_room
        .word   EmptyObjectState
        .word   EmptyObjectImages
        .byte   $00,$00,$00                                      ; Color, monochrome color, display size

; Unused cartridge padding.
        .byte   $00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00

; All cartridge vectors enter the same startup path.
        .word   CartridgeEntry
        .word   CartridgeEntry
        .word   CartridgeEntry
