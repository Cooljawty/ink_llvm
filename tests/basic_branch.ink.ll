; IO Functions
declare external i32 @puts(i8* nocapture) nounwind
declare external i32 @fputs(i8* nocapture, ptr nocapture) nounwind
@stdin  = extern_weak global %FILE_type
@stderr = extern_weak global %FILE_type
@stdout = extern_weak global  %FILE_type

; Memory allocation Functions
declare external ptr @malloc(i32)
declare external void @free(ptr)

%FILE_type =						type opaque
							; {up: ptr, ret: ptr}
%call_chain_type =			type { ptr, ptr }
							; {text: ptr, tags: ptr}
%choice_type =				type { ptr, ptr }
							;0: { choice_index: i32, 
							;1:	call_chain: {ptr, ptr}, 
							;2:	continue_flag: i1, 
							;3:	out_stream: ptr }
%promise_type =				type { i32, ptr, i1, ptr } 

%choice_list_type =		type {i32, ptr}
@choice_list = private global %choice_list_type { i32 0, ptr null }

;Runtime Functions, takes handel or null
define ptr @Step(ptr %story_handel) 
{
entry:
%new_instance =				icmp eq ptr %story_handel, null
							br i1 %new_instance, label %initilize, label %load_promise
initilize:
%init_message.addr =		getelementptr [7 x i8], ptr @init_message, i32 0, i32 0
							call i32 @puts(ptr %init_message.addr)
%new_instance_handel =		call ptr @__root()
							ret ptr %new_instance_handel

load_promise:
; %handel =					phi ptr [%new_instance_handel, %initilize], [%story_handel, %entry]
%handel =					phi ptr [%story_handel, %entry]

%promise.addr =				call ptr @llvm.coro.promise(ptr %handel, i32 4, i1 false) ; TODO: Get target platform alignment

%up_handel.addr =			getelementptr %promise_type, ptr %promise.addr, i32 1, i32 0
%up_handel =				load ptr, ptr %up_handel.addr

%ret_handel.addr =			getelementptr %promise_type, ptr %promise.addr, i32 1, i32 1
%ret_handel =				load ptr, ptr %ret_handel.addr

%outstream.addr =			getelementptr %promise_type, ptr %promise.addr, i32 3
							store %FILE_type @stdout, ptr %out_stream.addr

%end_of_knot =				call i1 @llvm.coro.done(ptr %handel)
							br i1 %end_of_knot, label %done, label %continuing
done:
							call void @llvm.coro.destroy(ptr %handel)
%diverting =				icmp ne ptr %up_handel, null
							br i1 %diverting, label %divert, label %done2
done2:
%return_from_tunnel =		icmp ne ptr %ret_handel, null
							br i1 %return_from_tunnel, label %call_ret, label %end
							divert:
							%has_return_handel =		icmp ne ptr %ret_handel, null
br i1 %has_return_handel, label %delete_chain, label %call_up
delete_chain:
%parent_handel =			phi ptr [%ret_handel, %divert], [%ret_chain_handel, %delete_chain]
;Getting ret handel
%ret_promise.addr =			call ptr @llvm.coro.promise(ptr %parent_handel, i32 4, i1 false) ; TODO: Get target platform alignment
%ret_chain_handel.addr =	getelementptr %promise_type, ptr %ret_promise.addr, i32 1, i32 1
%ret_chain_handel =			load ptr, ptr %ret_chain_handel.addr

							call void @llvm.coro.destroy(ptr %parent_handel)
;Looping
%end_of_chain =				icmp eq ptr %ret_chain_handel, null
							br i1 %end_of_chain, label %call_up, label %delete_chain

continuing:
%tunneling =				icmp ne ptr %up_handel, null
							br i1 %tunneling, label %call_up, label %resume
call_ret:
							br label %resume
call_up:
							br label %resume
resume:
%resume_handel =			phi ptr [%handel, %continuing], [%up_handel, %call_up], [%ret_handel, %call_ret]

%resume_promise.addr =		call ptr @llvm.coro.promise(ptr %handel, i32 4, i1 false) ; TODO: Get target platform alignment
%continue_flag.addr =		getelementptr %promise_type, ptr %resume_promise.addr, i32 2
%continue_flag =			load i1, ptr %continue_flag.addr
							br i1 %continue_flag, label %resume_call, label %resume_wait
resume_call:
							call i32 @puts(ptr @resume_message)

							call void @llvm.coro.resume(ptr %resume_handel)
							;TODO: call flush()
							ret ptr %resume_handel
resume_wait:
							ret ptr %resume_handel
end:
							ret ptr null
}

; Steps through the given Story handel returning all lines of content until
; the story reaches a choice point/end of story
; define ptr @ContinueMaximally(ptr %handel);

; Returns false if story requires a choice selection or otherwise cannot continue
; it's control flow
define i1 @CanContinue(ptr %handel)
{
%promise.addr =				call ptr @llvm.coro.promise(ptr %handel, i32 4, i1 false) ; TODO: Get target platform alignment

%continue_flag.addr =		getelementptr %promise_type, ptr %promise.addr, i32 2
%continue_flag =			load i1, ptr %continue_flag.addr
							ret i1 %continue_flag
}

; Returns the number of choices availabe from a given story handel
define i32 @ChoiceCount(ptr %handel)
{
%choice_count.addr =		getelementptr %choice_list_type, ptr @choice_list, i32 0
%choice_count =				load i32, ptr %choice_count.addr
							ret i32 %choice_count
}

; Returns a choice object at a given index from the given story handel
; define Choice @GetChoice(ptr %handel, i32);

; Selects the choice at a given index for the given story handel
; Note: Does not continue story
define void @ChooseChoiceIndex(ptr %handel, i32 %choice_index)
{
%choice_count.addr =		getelementptr %choice_list_type, ptr @choice_list, i32 0
%choice_count =				load i32, ptr %choice_count.addr

%index_less_than =			icmp uge i32 %choice_index, 0
%index_positive = 			icmp ult i32 %choice_index, %choice_count
%valid_index =				and i1 %index_less_than, %index_positive

							br i1 %valid_index, label %success, label %error
error:
							call i32 @puts(ptr @error_message)
							ret void
success:
%promise.addr =				call ptr @llvm.coro.promise(ptr %handel, i32 4, i1 false) ; TODO: Get target platform alignment
%promise.choice_index.addr =getelementptr %promise_type, ptr %promise.addr, i32 0
							store i32 %choice_index, ptr %promise.choice_index.addr
%promise.continue_flag.addr=getelementptr %promise_type, ptr %promise.addr, i32 2
							store i1 true, ptr %promise.continue_flag.addr
							ret void
}

; Story

;Content Strings
@story.str_0 =					constant [7 x i8] c"Hello!\00"
@story.str_1 =					constant [1 x i8] c"\00"

@story.choice_0.str_0 =			constant [7 x i8] c"Chose \00"
@story.choice_0.str_choice =	constant [2 x i8] c"A\00"
@story.choice_0.str_1 =			constant [11 x i8] c" the first\00"

@story.choice_1.str_0 =			constant [4 x i8] c"Or \00"
@story.choice_1.str_choice =	constant [2 x i8] c"B\00"
@story.choice_1.str_1 =			constant [12 x i8] c" the second\00"

@story.gather_0.str_0 =			constant [8 x i8] c"The end\00"

@error_message =				constant [7 x i8] c"Error!\00"
@debug_message =				constant [7 x i8] c"Debug!\00"
@init_message =					constant [6 x i8] c"init!\00"
@resume_message =				constant [8 x i8] c"resume!\00"

define ptr @__root() presplitcoroutine {
entry:
%promise =					alloca %promise_type
%id =						call token @llvm.coro.id(i32 4, ptr %promise, ptr null, ptr null) ;TODO: Determine native target alignment
%alloc_flag =				call i1 @llvm.coro.alloc(token %id)
							br i1 %alloc_flag, label %frame_alloc, label %begin
frame_alloc:
%size =						call i32 @llvm.coro.size.i32()
%alloc =					call ptr @malloc(i32 %size)
							br label %begin
begin:
%phi_alloc =				phi ptr [null, %entry], [%alloc, %frame_alloc]
%handel =					call noalias ptr @llvm.coro.begin(token %id, ptr %phi_alloc)

%choice_count.addr =		getelementptr %choice_list_type, ptr @choice_list, i32 0
%out_stream.addr =			getelementptr %promise_type, ptr %promise, i32 3
%out_stream =				load ptr, ptr %out_stream.addr

							br label %story

suspend:
%unused =					call i1 @llvm.coro.end(ptr null, i1 false, token none)
							ret ptr %handel
error:
							;TODO: Error handeling
							call i32 @puts(ptr @error_message)
							br label %suspend

destroy:
%frame_mem =				call ptr @llvm.coro.free(token %id, ptr %handel)
%free_frame =				icmp ne ptr %frame_mem, null
							br i1 %free_frame, label %frame_free, label %end
frame_free:
							call void @free(ptr %frame_mem)
							br label %end
end:
							ret ptr null

story:
%continue_flag.addr =		getelementptr %promise_type, ptr %promise, i32 2
							store i1 true, ptr %continue_flag.addr
							;"Hello!"
							call i32 @fputs(ptr @story.str_0, ptr %out_stream)

							;""
							call i32 @fputs(ptr @story.str_1, ptr %out_stream)

							br label %story.choice_point_0

story.choice_point_0:
							store i1 false, ptr %continue_flag.addr
							store i32 2, ptr %choice_count.addr

							%save_story.choice_point_0 = call token @llvm.coro.save(ptr %handel)
							%suspend_story.choice_point_0 = call i8 @llvm.coro.suspend(token %save_story.choice_point_0, i1 false)
							switch i8 %suspend_story.choice_point_0, label %suspend 
																	[i8 0, label %resume_story.choice_point_0
																	 i8 1, label %destroy]
resume_story.choice_point_0:
%choice_0_index.addr =		getelementptr %promise_type, ptr %promise, i32 0
%choice_0_index =			load i32, ptr %choice_0_index.addr
							switch i32 %choice_0_index, label %error [i32 0, label %story.choice_0 i32 1, label %story.choice_1]

story.choice_0:					;"* Chose [A] the first"
							store i1 true, ptr %continue_flag.addr

							;"Chose "
							call i32 @fputs(ptr @story.choice_0.str_0, ptr %out_stream)

							;" the first"
							call i32 @fputs(ptr @story.choice_0.str_1, ptr %out_stream)

							br label %story.gather_0

story.choice_1:					;"* Or [B] the second"
							store i1 true, ptr %continue_flag.addr

							;"Or "
							call i32 @fputs(ptr @story.choice_1.str_0, ptr %out_stream)

							;" the second"
%str_choice_1.1 =			getelementptr [0 x i8], ptr @story.choice_1.str_1, i32 0, i32 0
							call i32 @fputs(ptr @story.choice_1.str_1, ptr %out_stream)

							br label %story.gather_0

story.gather_0:					;"-"

							;"The end"
							call i32 @fputs(ptr @story.gather_0.str_0, ptr %out_stream)
							store i1 false, ptr %continue_flag.addr
							store i32 0, ptr %choice_count.addr

%suspend_story.gather_0.0 = call i8 @llvm.coro.suspend(token none, i1 true)
							switch i8 %suspend_story.gather_0.0, label %suspend [i8 0, label %error i8 1, label %destroy]
}
