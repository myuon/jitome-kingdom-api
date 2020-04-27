------------------------------- MODULE Janken -------------------------------
EXTENDS Naturals, Sequences, FiniteSets

CONSTANT NumClients, ServerStep
ASSUME NumClients \in Nat
ASSUME ServerStep \in Nat

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
        
    define
    {
        Drop(s,n) == SubSeq(s,n+1,Len(s))
    }

    macro new(c,v)
    {
        when Available[c] = TRUE;

        Available[c] := FALSE;
        Queue := Append(Queue, [data |-> v, client |-> c]);
        History := Append(History, [type |-> "new", data |-> v, client |-> c]);
    };
    
    macro do_battle(r1,r2)
    {
        \* r1: winner, r2: loserのとき
        if (\/ (r1.data = "rock" /\ r2.data = "scissors")
            \/ (r1.data = "paper" /\ r2.data = "rock")
            \/ (r1.data = "scissors" /\ r2.data = "paper"))
        {
            Available := [Available EXCEPT ![r1.client] = TRUE, ![r2.client] = TRUE];
            Queue := Drop(Queue, 2);
        \* r1: loser, r2: winnerのとき
        } else if (\/ (r1.data = "paper" /\ r2.data = "scissors")
            \/ (r1.data = "scissors" /\ r2.data = "rock")
            \/ (r1.data = "rock" /\ r2.data = "paper"))
        {
            Available := [Available EXCEPT ![r1.client] = TRUE, ![r2.client] = TRUE];
            Queue := Drop(Queue, 2);
        };
    };
    
    process (server = "server")
    variable count = 0;
    {
        server:
            while (count <= ServerStep) {
                either {
                    with (pair \in clients \times {"rock", "paper", "scissors"}) {
                        new(pair[1],pair[2]);
                    } 
                } or {
                    skip;
                };
                
                count := count + 1;
            };
    };
    
    process (worker \in {"1"})
    {
        worker:
            while (TRUE) {
                either {
                    \* 本来はTTLを設けてそれをチェックするが、TLA+ではStutteringがあるので気にしないことにする
                    await Len(Queue) >= 1;
                    Available[Queue[1].client] := TRUE;
                    Queue := Tail(Queue);
                } or {
                    await Len(Queue) >= 2;
                    do_battle(Queue[1], Queue[2]);
                } or {
                    skip;
                }
            };
    };
};
*)
\* BEGIN TRANSLATION - the hash of the PCal code: PCal-c1ecc6093ea8215e3b987c463155e525
\* Label server of process server at line 64 col 13 changed to server_
\* Label worker of process worker at line 80 col 13 changed to worker_
VARIABLES Available, History, Queue, pc

(* define statement *)
Drop(s,n) == SubSeq(s,n+1,Len(s))

VARIABLE count

vars == << Available, History, Queue, pc, count >>

ProcSet == {"server"} \cup ({"1"})

Init == (* Global variables *)
        /\ Available = [r \in clients |-> TRUE]
        /\ History = <<>>
        /\ Queue = <<>>
        (* Process server *)
        /\ count = 0
        /\ pc = [self \in ProcSet |-> CASE self = "server" -> "server_"
                                        [] self \in {"1"} -> "worker_"]

server_ == /\ pc["server"] = "server_"
           /\ IF count <= ServerStep
                 THEN /\ \/ /\ \E pair \in clients \times {"rock", "paper", "scissors"}:
                                 /\ Available[(pair[1])] = TRUE
                                 /\ Available' = [Available EXCEPT ![(pair[1])] = FALSE]
                                 /\ Queue' = Append(Queue, [data |-> (pair[2]), client |-> (pair[1])])
                                 /\ History' = Append(History, [type |-> "new", data |-> (pair[2]), client |-> (pair[1])])
                         \/ /\ TRUE
                            /\ UNCHANGED <<Available, History, Queue>>
                      /\ count' = count + 1
                      /\ pc' = [pc EXCEPT !["server"] = "server_"]
                 ELSE /\ pc' = [pc EXCEPT !["server"] = "Done"]
                      /\ UNCHANGED << Available, History, Queue, count >>

server == server_

worker_(self) == /\ pc[self] = "worker_"
                 /\ \/ /\ Len(Queue) >= 1
                       /\ Available' = [Available EXCEPT ![Queue[1].client] = TRUE]
                       /\ Queue' = Tail(Queue)
                    \/ /\ Len(Queue) >= 2
                       /\ IF \/ ((Queue[1]).data = "rock" /\ (Queue[2]).data = "scissors")
                             \/ ((Queue[1]).data = "paper" /\ (Queue[2]).data = "rock")
                             \/ ((Queue[1]).data = "scissors" /\ (Queue[2]).data = "paper")
                             THEN /\ Available' = [Available EXCEPT ![(Queue[1]).client] = TRUE, ![(Queue[2]).client] = TRUE]
                                  /\ Queue' = Drop(Queue, 2)
                             ELSE /\ IF        \/ ((Queue[1]).data = "paper" /\ (Queue[2]).data = "scissors")
                                        \/ ((Queue[1]).data = "scissors" /\ (Queue[2]).data = "rock")
                                        \/ ((Queue[1]).data = "rock" /\ (Queue[2]).data = "paper")
                                        THEN /\ Available' = [Available EXCEPT ![(Queue[1]).client] = TRUE, ![(Queue[2]).client] = TRUE]
                                             /\ Queue' = Drop(Queue, 2)
                                        ELSE /\ TRUE
                                             /\ UNCHANGED << Available, Queue >>
                    \/ /\ TRUE
                       /\ UNCHANGED <<Available, Queue>>
                 /\ pc' = [pc EXCEPT ![self] = "worker_"]
                 /\ UNCHANGED << History, count >>

worker(self) == worker_(self)

Next == server
           \/ (\E self \in {"1"}: worker(self))

Spec == Init /\ [][Next]_vars

\* END TRANSLATION - the hash of the generated TLA code (remove to silence divergence warnings): TLA-0b43c33e7d8acfb39c009d2b6ee176f3

-----------------------------------------------------------------------------

AtMostOneInQueue == (\A c \in clients:
    LET IsClientEq(q) == q.client = c IN
    Len(SelectSeq(Queue, IsClientEq)) <= 1
)

Safety == []AtMostOneInQueue

=============================================================================

\* Modification History
\* Last modified Mon Apr 27 23:52:07 JST 2020 by ioijoi
\* Created Mon Apr 27 16:16:15 JST 2020 by ioijoi
