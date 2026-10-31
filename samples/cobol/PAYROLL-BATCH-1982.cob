       IDENTIFICATION DIVISION.
       PROGRAM-ID. PAYBATCH.
      *================================================================
      * WEEKLY PAYROLL BATCH.  CONSOLIDATED MILLS, 1982.
      * PROCESSES EMPLOYEE RECORDS UNTIL THE END-OF-FILE FLAG IS SET.
      *================================================================
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       77  WS-HOURS            PIC 9(3)V99    VALUE 44.00.
       77  WS-PAY-RATE         PIC 9(3)V99    VALUE 12.50.
       77  WS-EMP-COUNT        PIC 999        VALUE 3.
       01  WS-RECORD.
           05  WS-OT-HOURS     PIC 9(3)V99.
           05  WS-GROSS        PIC 9(6)V99    COMP-3.
           05  WS-TAX          PIC 9(5)V99    COMP-3.
           05  WS-NET          PIC 9(6)V99    COMP-3.
           05  WS-EOF          PIC 9          VALUE 0.
           05  WS-PROCESSED    PIC 999.
           05  WS-TOTAL-NET    PIC 9(8)V99    COMP-3.
           05  WS-HIRE-YY      PIC 99         VALUE 82.
           05  WS-SERVICE-YRS  PIC 99.
       PROCEDURE DIVISION.
       MAIN-PARA.
           PERFORM PROCESS-EMPLOYEE UNTIL WS-EOF = 1.
           STOP RUN.
       PROCESS-EMPLOYEE.
           IF WS-HOURS > 40
               COMPUTE WS-OT-HOURS = WS-HOURS - 40
           ELSE
               MOVE 0 TO WS-OT-HOURS
           END-IF.
           COMPUTE WS-GROSS = (WS-HOURS - WS-OT-HOURS) * WS-PAY-RATE
               + WS-OT-HOURS * WS-PAY-RATE * 1.5.
           COMPUTE WS-TAX = WS-GROSS * 0.2.
           COMPUTE WS-NET = WS-GROSS - WS-TAX.
           ADD WS-NET TO WS-TOTAL-NET.
           ADD 1 TO WS-PROCESSED.
      * THE FLAG IS MEANT TO BE SET WHEN THE LAST RECORD IS DONE,
      * BUT THE PARAGRAPH THAT SETS IT IS NEVER PERFORMED.
       END-OF-BATCH.
           MOVE 1 TO WS-EOF.
           COMPUTE WS-SERVICE-YRS = 26 - WS-HIRE-YY.
           DISPLAY 'BATCH COMPLETE'.
