       IDENTIFICATION DIVISION.
       PROGRAM-ID. WIRECLR.
      *================================================================
      * WIRE TRANSFER CLEARING.  METRO CLEARING HOUSE, 1985.
      * CLEARS ONE BATCH OF WIRES, CHARGING A FEE ON EACH, THEN SETTLES.
      *================================================================
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       77  WS-WIRE-AMT         PIC 9(9)V99    VALUE 250000.00.
       77  WS-FEE-RATE         PIC V9999      VALUE 0.0015.
       77  WS-BATCH-SIZE       PIC 99         VALUE 50.
       01  WS-CLEARING.
           05  WS-SEQ          PIC 99.
           05  WS-FEE          PIC 9(7)V99    COMP-3.
           05  WS-QUEUE-DEPTH  PIC 9(5).
           05  WS-VALUE-YY     PIC 99         VALUE 85.
           05  WS-SETTLE-AMT   PIC 9(9)V99.
           05  WS-TOTAL-FEES   PIC 9(9)V99    COMP-3.
       PROCEDURE DIVISION.
       MAIN-PARA.
           IF WS-WIRE-AMT = 0
               GO TO EMPTY-BATCH
           END-IF.
           MOVE 1 TO WS-SEQ.
      * WS-SEQ IS PIC 99.  WHEN THE BATCH SIZE IS 99 THE SEQUENCE
      * WRAPS FROM 99 TO 0 AND IS NEVER GREATER THAN THE BATCH SIZE.
           PERFORM CLEAR-ONE-WIRE UNTIL WS-SEQ > WS-BATCH-SIZE.
           PERFORM SETTLE-BATCH.
           STOP RUN.
       CLEAR-ONE-WIRE.
           COMPUTE WS-FEE = WS-WIRE-AMT * WS-FEE-RATE.
           ADD WS-FEE TO WS-TOTAL-FEES.
      * EVERY WIRE IS QUEUED FOR RECONCILIATION; NOTHING EVER DEQUEUES.
           ADD 1 TO WS-QUEUE-DEPTH.
           ADD 1 TO WS-SEQ.
       SETTLE-BATCH.
           COMPUTE WS-SETTLE-AMT = WS-WIRE-AMT - WS-FEE.
       EMPTY-BATCH.
           STOP RUN.
       RECONCILE-QUEUE.
      * THE RECONCILIATION STEP WAS DROPPED FROM MAIN-PARA IN 1987.
           SUBTRACT 1 FROM WS-QUEUE-DEPTH.
           DISPLAY 'RECONCILED'.
