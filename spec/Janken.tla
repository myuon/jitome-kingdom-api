------------------------------- MODULE Janken -------------------------------
EXTENDS Naturals, Sequences

CONSTANT NumClients
ASSUME NumClients \in Nat

clients == 1..NumClients

Hand == {"rock", "paper", "scissors"}

Operations == [type: {"new"}, data: Hand, client: Nat]

(*
--algorithm janken_process
variables
    (* クライアントの状態, じゃんけんが送信できる状態か否か *)
    Available = [r \in clients |-> TRUE];
    
    (* operationの履歴 *)
    History = <<>>;
    
    (* 処理されていないハンドたち *)
    Queue = <<>>;
    
macro new(c,v)
begin
    when Available[c] = TRUE;
    Available[c] := FALSE;
    Queue := Append(Queue, [data |-> v, client |-> c]);
    History := Append(History, [type |-> "new", data |-> v, client |-> c]);
end macro;

process server = "server"
begin
    start_action:
        while TRUE do
            server_action:
                either
                    with pair \in clients \times {"rock", "paper", "scissors"} do
                        new(pair[1],pair[2]);
                    end with; 
                or
                    skip;
                end either;
        end while;
end process;

process worker \in {"1"}
begin
    process_hands:
        while TRUE do
            worker_action:
                if Len(Queue) >= 2 then
                    Queue := Tail(Tail(Queue))
                end if;
        end while;
end process;

end algorithm;
*)
\* BEGIN TRANSLATION - the hash of the PCal code: PCal-5812d94535626b3fe2f71643c9fe7546
VARIABLES Available, History, Queue, pc

vars == << Available, History, Queue, pc >>

ProcSet == {"server"} \cup ({"1"})

Init == (* Global variables *)
        /\ Available = [r \in clients |-> TRUE]
        /\ History = <<>>
        /\ Queue = <<>>
        /\ pc = [self \in ProcSet |-> CASE self = "server" -> "start_action"
                                        [] self \in {"1"} -> "process_hands"]

start_action == /\ pc["server"] = "start_action"
                /\ pc' = [pc EXCEPT !["server"] = "server_action"]
                /\ UNCHANGED << Available, History, Queue >>

server_action == /\ pc["server"] = "server_action"
                 /\ \/ /\ \E pair \in clients \times {"rock", "paper", "scissors"}:
                            /\ Available[(pair[1])] = TRUE
                            /\ Available' = [Available EXCEPT ![(pair[1])] = FALSE]
                            /\ Queue' = Append(Queue, [data |-> (pair[2]), client |-> (pair[1])])
                            /\ History' = Append(History, [type |-> "new", data |-> (pair[2]), client |-> (pair[1])])
                    \/ /\ TRUE
                       /\ UNCHANGED <<Available, History, Queue>>
                 /\ pc' = [pc EXCEPT !["server"] = "start_action"]

server == start_action \/ server_action

process_hands(self) == /\ pc[self] = "process_hands"
                       /\ pc' = [pc EXCEPT ![self] = "worker_action"]
                       /\ UNCHANGED << Available, History, Queue >>

worker_action(self) == /\ pc[self] = "worker_action"
                       /\ IF Len(Queue) >= 2
                             THEN /\ Queue' = Tail(Tail(Queue))
                             ELSE /\ TRUE
                                  /\ Queue' = Queue
                       /\ pc' = [pc EXCEPT ![self] = "process_hands"]
                       /\ UNCHANGED << Available, History >>

worker(self) == process_hands(self) \/ worker_action(self)

Next == server
           \/ (\E self \in {"1"}: worker(self))

Spec == Init /\ [][Next]_vars

\* END TRANSLATION - the hash of the generated TLA code (remove to silence divergence warnings): TLA-95cb7062661d6bca04092b820bd52d79

=============================================================================
\* Modification History
\* Last modified Mon Apr 27 18:39:24 JST 2020 by ioijoi
\* Created Mon Apr 27 16:16:15 JST 2020 by ioijoi
