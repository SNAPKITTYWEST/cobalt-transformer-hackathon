*> =====================================================================
*> PROGRAM : BANKOO
*> PURPOSE : OO COBOL — class, methods, constructor, inheritance-ish
*> COMPILE : cobc -x -std=ibm -free -Wall BANKOO.cob
*> =====================================================================
identification division.
program-id. BANKOO.
environment division.
configuration section.
repository.
class BankAccount as "bankaccount"
class SavingsAccount as "savingsaccount".
data division.
working-storage section.
01 acct object reference BankAccount.
01 sav object reference SavingsAccount.
01 ws-balance pic s9(9)v99.
procedure division.
main.
*> ---------- instantiate + use base class ----------------------------
invoke BankAccount "new" using
by content 00000001
by content "Alice Smith"
returning acct
invoke acct "deposit" using by content 001000.00
invoke acct "deposit" using by content 000250.00
invoke acct "withdraw" using by content 000100.00
invoke acct "getBalance" returning ws-balance
display "Alice balance : " ws-balance
*> ---------- subclass with overriding method --------------------------
invoke SavingsAccount "new" using
by content 00000002
by content "Bob Jones"
by content 000005.00
returning sav
invoke sav "deposit" using by content 000500.00
invoke sav "withdraw" using by content 000600.00 *> blocked: min balance
invoke sav "getBalance" returning ws-balance
display "Bob balance : " ws-balance
invoke acct "finalize" returning acct *> destructor
invoke sav "finalize" returning sav
display "OO COBOL DONE"
goback
.
end program BANKOO.
*> =====================================================================
*> CLASS : BankAccount
*> =====================================================================
class-id. BankAccount.
environment division.
configuration section.
repository.
data division.
working-storage section.
01 ws-number pic 9(8).
01 ws-name pic x(20).
01 ws-balance pic s9(9)v99 value zero.
procedure division.
method-id. "new" constructor.
data division.
linkage section.
01 ln-number pic 9(8).
01 ln-name pic x(20).
procedure division using ln-number ln-name
returning self.
move ln-number to ws-number
move ln-name to ws-name
display " [ctor] account " ws-number " created for " ws-name
goback
.
method-id. "deposit".
data division.
linkage section.
01 ln-amount pic s9(9)v99.
procedure division using ln-amount.
if ln-amount > zero
add ln-amount to ws-balance
display " [deposit] +" ln-amount " -> " ws-balance
else
display " [deposit] REJECTED non-positive amount"
end-if
goback
.
method-id. "withdraw".
data division.
linkage section.
01 ln-amount pic s9(9)v99.
procedure division using ln-amount.
if ln-amount <= zero
display " [withdraw] REJECTED non-positive amount"
else if ln-amount > ws-balance
display " [withdraw] REJECTED insufficient funds"
else
subtract ln-amount from ws-balance
display " [withdraw] -" ln-amount " -> " ws-balance
end-if
goback
.
method-id. "getBalance".
data division.
linkage section.
01 ln-balance pic s9(9)v99.
procedure division returning ln-balance.
move ws-balance to ln-balance
goback
.
method-id. "finalize" final.
procedure division returning self.
display " [dtor] account " ws-number " destroyed"
goback
.
end class BankAccount.
*> =====================================================================
*> CLASS : SavingsAccount — inherits BankAccount, overrides withdraw
*> =====================================================================
class-id. SavingsAccount inheriting from BankAccount.
environment division.
configuration section.
repository.
data division.
working-storage section.
01 ws-minimum pic s9(9)v99.
procedure division.
method-id. "new" constructor.
data division.
linkage section.
01 ln-number pic 9(8).
01 ln-name pic x(20).
01 ln-minimum pic s9(9)v99.
procedure division using ln-number ln-name ln-minimum
returning self.
invoke super "new" using ln-number ln-name
move ln-minimum to ws-minimum
display " [ctor] SAVINGS minimum=" ws-minimum
goback
.
method-id. "withdraw" overriding.
data division.
linkage section.
01 ln-amount pic s9(9)v99.
01 ln-balance pic s9(9)v99.
procedure division using ln-amount.
invoke self "getBalance" returning ln-balance
subtract ln-amount from ln-balance
if ln-balance >= ws-minimum
invoke super "withdraw" using ln-amount
else
display " [withdraw] BLOCKED: would breach minimum balance"
end-if
goback
.
end class SavingsAccount.
