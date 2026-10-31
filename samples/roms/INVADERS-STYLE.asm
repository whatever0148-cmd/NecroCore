; INVADERS-STYLE: a row of four invaders marches and descends; the player shoots.
; Keys: 4 / 6 move, 5 fires. Win by clearing the row, lose when they reach y 24.
; Original program written for NecroCore (2026). Not a historical binary.
; Registers: V2 player x, V4/V5 shot x/y, V6 shot active, V7 invader base x,
;            V8 alive mask (bits 0..3), V9 direction, VA invader y, VB frame counter,
;            VC shot speed, VD scratch, VE scratch, V0/V1 scratch.
start:
    CLS
    LD V2, 28
    LD V6, 0
    LD V7, 8
    LD V8, 0x0F
    LD V9, 1
    LD VA, 2
    LD VB, 0
    .patch SHOT_SPEED rows the shot climbs per frame
    LD VC, 2
loop:
    CLS
    ; player input
    LD V0, 4
    SKNP V0
    ADD V2, 0xFF
    LD V0, 6
    SKNP V0
    ADD V2, 1
    SNE V2, 0xFF
    LD V2, 0
    SNE V2, 57
    LD V2, 56
    ; fire
    LD V0, 5
    SKNP V0
    CALL fire
    ; move the shot
    SE V6, 1
    JP skip_shot
    SUB V5, VC
    SE VF, 1
    LD V6, 0
    CALL check_hit
skip_shot:
    ; march the invaders every 8th frame
    ADD VB, 1
    LD V0, VB
    LD V1, 7
    AND V0, V1
    SE V0, 0
    JP draw
    ADD V7, V9
    SNE V7, 32
    CALL turn
    SNE V7, 0
    CALL turn
draw:
    LD I, player_sprite
    LD V0, 28
    DRW V2, V0, 3
    SE V6, 1
    JP draw_invaders
    LD I, shot_sprite
    DRW V4, V5, 2
draw_invaders:
    LD I, invader_sprite
    LD V1, V7
    LD VD, 1
    AND VD, V8
    SE VD, 0
    DRW V1, VA, 4
    ADD V1, 8
    LD VD, 2
    AND VD, V8
    SE VD, 0
    DRW V1, VA, 4
    ADD V1, 8
    LD VD, 4
    AND VD, V8
    SE VD, 0
    DRW V1, VA, 4
    ADD V1, 8
    LD VD, 8
    AND VD, V8
    SE VD, 0
    DRW V1, VA, 4
    ; end conditions
    SNE V8, 0
    JP win
    LD V0, VA
    LD V1, 24
    SUB V0, V1
    SE VF, 0
    JP lose
    LD V0, 1
    LD DT, V0
wait:
    LD V0, DT
    SE V0, 0
    JP wait
    JP loop
fire:
    SNE V6, 1
    RET
    LD V6, 1
    LD V4, V2
    ADD V4, 2
    LD V5, 26
    RET
turn:
    ; reverse direction and drop one row
    LD V0, 0
    SUBN V9, V0
    .patch DESCENT rows the row drops at each wall
    ADD VA, 2
    RET
check_hit:
    ; shot at the invader row?  V5 must be within VA..VA+3
    LD V0, V5
    SUB V0, VA
    SE VF, 1
    RET
    LD V1, 3
    SUB V1, V0
    SE VF, 1
    RET
    ; column = (shot x - base x) / 8, must be 0..3
    LD V0, V4
    SUB V0, V7
    SE VF, 1
    RET
    SHR V0
    SHR V0
    SHR V0
    LD V1, 3
    SUB V1, V0
    SE VF, 1
    RET
    LD I, bit_table
    ADD I, V0
    LD V0, [I]
    LD V1, V0
    AND V1, V8
    SE V1, 0
    XOR V8, V0
    SE V1, 0
    LD V6, 0
    RET
win:
    JP win
lose:
    JP lose
player_sprite:
    DB 0x20, 0x70, 0xF8
shot_sprite:
    DB 0x80, 0x80
invader_sprite:
    DB 0x5A, 0x3C, 0x7E, 0x24
bit_table:
    DB 0x01, 0x02, 0x04, 0x08
