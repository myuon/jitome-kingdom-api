------------------------------- MODULE Janken -------------------------------
EXTENDS Naturals, Sequences, FiniteSets

CONSTANT NumClients
ASSUME NumClients \in Nat

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
            Queue := Tail(Tail(Queue));
        \* r1: loser, r2: winnerのとき
        } else if (\/ (r1.data = "paper" /\ r2.data = "scissors")
            \/ (r1.data = "scissors" /\ r2.data = "rock")
            \/ (r1.data = "rock" /\ r2.data = "paper"))
        {
            Available := [Available EXCEPT ![r1.client] = TRUE, ![r2.client] = TRUE];
            Queue := Tail(Tail(Queue));
        };
    };
    
    process (server = "server")
    {
        server:
            either {
                with (pair \in clients \times {"rock", "paper", "scissors"}) {
                    new(pair[1],pair[2]);
                } 
            } or {
                skip;
            }
    };
    
    process (worker \in {"1"})
    {
        worker:
            while (TRUE) {
                either {
                    when Len(Queue) >= 2;
                    do_battle(Queue[1], Queue[2]);
                } or {
                    skip;
                }
            };
    };
};
*)
\* BEGIN TRANSLATION - the hash of the PCal code: PCal-cde9d4a5150c466d154c58c123c93017
\* Label server of process server at line 57 col 13 changed to server_
\* Label worker of process worker at line 69 col 13 changed to worker_
VARIABLES Available, History, Queue, pc

vars == << Available, History, Queue, pc >>

ProcSet == {"server"} \cup ({"1"})

Init == (* Global variables *)
        /\ Available = [r \in clients |-> TRUE]
        /\ History = <<>>
        /\ Queue = <<>>
        /\ pc = [self \in ProcSet |-> CASE self = "server" -> "server_"
                                        [] self \in {"1"} -> "worker_"]

server_ == /\ pc["server"] = "server_"
           /\ \/ /\ \E pair \in clients \times {"rock", "paper", "scissors"}:
                      /\ Available[(pair[1])] = TRUE
                      /\ Available' = [Available EXCEPT ![(pair[1])] = FALSE]
                      /\ Queue' = Append(Queue, [data |-> (pair[2]), client |-> (pair[1])])
                      /\ History' = Append(History, [type |-> "new", data |-> (pair[2]), client |-> (pair[1])])
              \/ /\ TRUE
                 /\ UNCHANGED <<Available, History, Queue>>
           /\ pc' = [pc EXCEPT !["server"] = "Done"]

server == server_

worker_(self) == /\ pc[self] = "worker_"
                 /\ \/ /\ Len(Queue) >= 2
                       /\ IF \/ ((Queue[1]).data = "rock" /\ (Queue[2]).data = "scissors")
                             \/ ((Queue[1]).data = "paper" /\ (Queue[2]).data = "rock")
                             \/ ((Queue[1]).data = "scissors" /\ (Queue[2]).data = "paper")
                             THEN /\ Available' = [Available EXCEPT ![(Queue[1]).client] = TRUE, ![(Queue[2]).client] = TRUE]
                                  /\ Queue' = Tail(Tail(Queue))
                             ELSE /\ IF        \/ ((Queue[1]).data = "paper" /\ (Queue[2]).data = "scissors")
                                        \/ ((Queue[1]).data = "scissors" /\ (Queue[2]).data = "rock")
                                        \/ ((Queue[1]).data = "rock" /\ (Queue[2]).data = "paper")
                                        THEN /\ Available' = [Available EXCEPT ![(Queue[1]).client] = TRUE, ![(Queue[2]).client] = TRUE]
                                             /\ Queue' = Tail(Tail(Queue))
                                        ELSE /\ TRUE
                                             /\ UNCHANGED << Available, Queue >>
                    \/ /\ TRUE
                       /\ UNCHANGED <<Available, Queue>>
                 /\ pc' = [pc EXCEPT ![self] = "worker_"]
                 /\ UNCHANGED History

worker(self) == worker_(self)

Next == server
           \/ (\E self \in {"1"}: worker(self))

Spec == Init /\ [][Next]_vars

\* END TRANSLATION - the hash of the generated TLA code (remove to silence divergence warnings): TLA-891cb3fc5887dc2bd33c71a7b08f4ab3

-----------------------------------------------------------------------------

AtMostOneInQueue == (\A c \in clients:
    LET IsClientEq(q) == q.client = c IN
    Len(SelectSeq(Queue, IsClientEq)) <= 1
)

Soundness == []AtMostOneInQueue

=============================================================================

\* Modification History
\* Last modified Mon Apr 27 20:51:53 JST 2020 by ioijoi
\* Created Mon Apr 27 16:16:15 JST 2020 by ioijoi
