------------------------------- MODULE Janken -------------------------------
EXTENDS Naturals, Sequences, FiniteSets

CONSTANT NumClients, ServerStep, NumWorker
ASSUME NumClients \in Nat
ASSUME ServerStep \in Nat
ASSUME NumWorker \in Nat

clients == 1..NumClients

Hand == {"rock", "paper", "scissors"}

Operations == [type: {"new"}, data: Hand, client: Nat]

(*
--algorithm janken_process
{
    variables
        (* クライアントの状態, じゃんけんが送信できる状態か否か *)
        Available = [r \in clients |-> TRUE];
        
        (* operationの履歴 *)
        History = <<>>;
        
        (* 処理されていないハンドたち *)
        Queue = <<>>;

        (* DBのロックを検証するためのもの *)        
        EventRecordLock = <<>>;

    define
    {
        Drop(s,n) == SubSeq(s,n+1,Len(s))
        
        isWin(r1,r2) ==
            \/ (r1.data = "rock" /\ r2.data = "scissors")
            \/ (r1.data = "paper" /\ r2.data = "rock")
            \/ (r1.data = "scissors" /\ r2.data = "paper")

        isLose(r1,r2) ==
            \/ (r1.data = "paper" /\ r2.data = "scissors")
            \/ (r1.data = "scissors" /\ r2.data = "rock")
            \/ (r1.data = "rock" /\ r2.data = "paper")
    }

    macro new(c,v)
    {
        when Available[c] = TRUE;

        Available[c] := FALSE;
        History := Append(History, [type |-> "new", data |-> v, client |-> c]);
        EventRecordLock := Append(EventRecordLock, TRUE);
        Queue := Append(Queue, [data |-> v, client |-> c, id |-> Len(History)]);
    };
    
    macro acquire_record_lock(r1,r2)
    {
        await EventRecordLock[r1.id] /\ EventRecordLock[r2.id];
        EventRecordLock := [EventRecordLock EXCEPT ![r1.id] = FALSE, ![r2.id] = FALSE];
    };
    
    macro release_lock(r1,r2)
    {
        EventRecordLock := [EventRecordLock EXCEPT ![r1.id] = TRUE, ![r2.id] = TRUE];
    };
    
    process (server = 0)
    variable count = 0;
    {
        server:
            while (count <= ServerStep) {
                with (pair \in clients \times {"rock", "paper", "scissors"}) {
                    new(pair[1],pair[2]);
                };
                
                count := count + 1;
            };
    };
    
    process (worker \in 1..NumWorker)
    {
        worker:
            while (TRUE) {
                either {
                    \* 本来はTTLを設けてそれをチェックするが、TLA+ではStutteringがあるので気にしないことにする
                    await Len(Queue) >= 1;
                    Available[Queue[1].client] := TRUE;
                    
                    \* ロックを取って書き込みを行う
                    await EventRecordLock[Queue[1].id] = TRUE;

                    Queue := Tail(Queue);
                } or {
                    await Len(Queue) >= 2;
                    
                    if (isWin(Queue[1], Queue[2]) \/ isLose(Queue[1], Queue[2])) {
                        \* ロックを取って書き込みを行う
                        acquire_record_lock(Queue[1], Queue[2]);

                        lock_released:
                            release_lock(Queue[1], Queue[2]);
                            Available := [Available EXCEPT ![Queue[1].client] = TRUE, ![Queue[2].client] = TRUE];
                            Queue := Drop(Queue, 2);
                    }
                } or {
                    skip;
                }
            };
    };
};
*)
\* BEGIN TRANSLATION - the hash of the PCal code: PCal-dd837a185783166ba8c08ddd9fd25646
\* Label server of process server at line 71 col 13 changed to server_
\* Label worker of process worker at line 83 col 13 changed to worker_
VARIABLES Available, History, Queue, EventRecordLock, pc

(* define statement *)
Drop(s,n) == SubSeq(s,n+1,Len(s))

isWin(r1,r2) ==
    \/ (r1.data = "rock" /\ r2.data = "scissors")
    \/ (r1.data = "paper" /\ r2.data = "rock")
    \/ (r1.data = "scissors" /\ r2.data = "paper")

isLose(r1,r2) ==
    \/ (r1.data = "paper" /\ r2.data = "scissors")
    \/ (r1.data = "scissors" /\ r2.data = "rock")
    \/ (r1.data = "rock" /\ r2.data = "paper")

VARIABLE count

vars == << Available, History, Queue, EventRecordLock, pc, count >>

ProcSet == {0} \cup (1..NumWorker)

Init == (* Global variables *)
        /\ Available = [r \in clients |-> TRUE]
        /\ History = <<>>
        /\ Queue = <<>>
        /\ EventRecordLock = <<>>
        (* Process server *)
        /\ count = 0
        /\ pc = [self \in ProcSet |-> CASE self = 0 -> "server_"
                                        [] self \in 1..NumWorker -> "worker_"]

server_ == /\ pc[0] = "server_"
           /\ IF count <= ServerStep
                 THEN /\ \E pair \in clients \times {"rock", "paper", "scissors"}:
                           /\ Available[(pair[1])] = TRUE
                           /\ Available' = [Available EXCEPT ![(pair[1])] = FALSE]
                           /\ History' = Append(History, [type |-> "new", data |-> (pair[2]), client |-> (pair[1])])
                           /\ EventRecordLock' = Append(EventRecordLock, TRUE)
                           /\ Queue' = Append(Queue, [data |-> (pair[2]), client |-> (pair[1]), id |-> Len(History')])
                      /\ count' = count + 1
                      /\ pc' = [pc EXCEPT ![0] = "server_"]
                 ELSE /\ pc' = [pc EXCEPT ![0] = "Done"]
                      /\ UNCHANGED << Available, History, Queue, 
                                      EventRecordLock, count >>

server == server_

worker_(self) == /\ pc[self] = "worker_"
                 /\ \/ /\ Len(Queue) >= 1
                       /\ Available' = [Available EXCEPT ![Queue[1].client] = TRUE]
                       /\ EventRecordLock[Queue[1].id] = TRUE
                       /\ Queue' = Tail(Queue)
                       /\ pc' = [pc EXCEPT ![self] = "worker_"]
                       /\ UNCHANGED EventRecordLock
                    \/ /\ Len(Queue) >= 2
                       /\ IF isWin(Queue[1], Queue[2]) \/ isLose(Queue[1], Queue[2])
                             THEN /\ EventRecordLock[(Queue[1]).id] /\ EventRecordLock[(Queue[2]).id]
                                  /\ EventRecordLock' = [EventRecordLock EXCEPT ![(Queue[1]).id] = FALSE, ![(Queue[2]).id] = FALSE]
                                  /\ pc' = [pc EXCEPT ![self] = "lock_released"]
                             ELSE /\ pc' = [pc EXCEPT ![self] = "worker_"]
                                  /\ UNCHANGED EventRecordLock
                       /\ UNCHANGED <<Available, Queue>>
                    \/ /\ TRUE
                       /\ pc' = [pc EXCEPT ![self] = "worker_"]
                       /\ UNCHANGED <<Available, Queue, EventRecordLock>>
                 /\ UNCHANGED << History, count >>

lock_released(self) == /\ pc[self] = "lock_released"
                       /\ EventRecordLock' = [EventRecordLock EXCEPT ![(Queue[1]).id] = TRUE, ![(Queue[2]).id] = TRUE]
                       /\ Available' = [Available EXCEPT ![Queue[1].client] = TRUE, ![Queue[2].client] = TRUE]
                       /\ Queue' = Drop(Queue, 2)
                       /\ pc' = [pc EXCEPT ![self] = "worker_"]
                       /\ UNCHANGED << History, count >>

worker(self) == worker_(self) \/ lock_released(self)

Next == server
           \/ (\E self \in 1..NumWorker: worker(self))

Spec == Init /\ [][Next]_vars

\* END TRANSLATION - the hash of the generated TLA code (remove to silence divergence warnings): TLA-cd09c3137d6213c5be88dec9c5bd5b14

-----------------------------------------------------------------------------

AtMostOneInQueue == (\A c \in clients:
    LET IsClientEq(q) == q.client = c IN
    Len(SelectSeq(Queue, IsClientEq)) <= 1
)

Safety == []AtMostOneInQueue

=============================================================================

\* Modification History
\* Last modified Tue Apr 28 00:36:26 JST 2020 by ioijoi
\* Created Mon Apr 27 16:16:15 JST 2020 by ioijoi
