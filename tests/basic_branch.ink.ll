; IO Functions
declare external i32 @puts(i8* nocapture) nounwind

; Memory allocation Functions
declare external ptr @malloc(i32)
declare external void @free(ptr)

%FILE_type =					type opaque

%choice_type =					type { ptr, ptr }
								; 0 text: ptr
								; 1 tags: ptr

%choice_list_type =				type {i32, ptr}
@choice_list =					private global %choice_list_type { i32 0, ptr null }

%story_handel_type =			type {ptr, ptr}
								; 0 continuation_fn: fn*
								; 1 frame_buffer: ptr

; Message strings
@newline_str =					constant [2 x i8] c"\0A\00"
@error_message =				constant [7 x i8] c"Error!\00"
@debug_message =				constant [7 x i8] c"Debug!\00"
@debug_up_message =				constant [11 x i8] c"calling up\00"
@debug_ret_message =			constant [12 x i8] c"calling ret\00"
@init_message =					constant [6 x i8] c"init!\00"
@resume_message =				constant [8 x i8] c"resume!\00"

; Strings
%string_type = type {ptr, i32}
declare extern_weak ptr @new_string()
declare extern_weak void @free_string(ptr nocapture)
declare extern_weak i32 @write_string(ptr nocapture, i8* nocapture)
declare extern_weak i32 @read_string(ptr nocapture, i8* nocapture)
declare extern_weak void @flush_string(ptr nocapture)

@out_stream = private global %string_type {ptr null, i32 0}

;Runtime Variables
@continue_maximally =		internal global i1 false

;Runtime Functions, takes handel or null
define ptr @Step(ptr %handel.address)
{
resume:
							call i32 @puts(ptr @resume_message)
%output_string.addr =		call ptr @new_string()
%output_string =			load %string_type, ptr %output_string.addr
							store %string_type %output_string, ptr @out_stream

%cont_fn.addr =				getelementptr %story_handel_type, ptr %handel.address, i32 0
%cont_fn =					load ptr, ptr %cont_fn.addr
%frame_buffer.addr =		getelementptr %story_handel_type, ptr %handel.address, i32 1
%frame_buffer =				load ptr, ptr %frame_buffer.addr

							store i1 false, ptr @continue_maximally

%result =					call {ptr, %yield_type} %cont_fn(ptr %frame_buffer, i1 0)
							
%result.cont_fn =			extractvalue {ptr, %yield_type} %result, 0
							store ptr %result.cont_fn, ptr %cont_fn.addr

							ret ptr %output_string.addr
}

; Initilizes a new instance of a ink story. Returing pointer to handel
define ptr @NewStory()
{
entry:
							call i32 @puts(ptr @init_message)

%ptr_size =					load i32, ptr @ptr_size
%handel_size =				mul i32 %ptr_size, 2 
%frame_buffer =				call ptr @malloc(i32 %ptr_size)
%handel =					call ptr @malloc(i32 %handel_size)

%cont_fn.address =			getelementptr %story_handel_type, ptr %handel, i32 0	
%frame_buffer.address =		getelementptr %story_handel_type, ptr %handel, i32 1
							store ptr @__root, ptr %cont_fn.address
							store ptr %frame_buffer, ptr %frame_buffer.address

							ret ptr %handel
}

; Steps through the given Story handel returning all lines of content until
; the story reaches a choice point/end of story
define ptr @ContinueMaximally(ptr %handel.address)
{
resume:
							call i32 @puts(ptr @resume_message)
%output_string.addr =		call ptr @new_string()
%output_string =			load %string_type, ptr %output_string.addr
							store %string_type %output_string, ptr @out_stream

%cont_fn.addr =				getelementptr %story_handel_type, ptr %handel.address, i32 0
%cont_fn =					load ptr, ptr %cont_fn.addr
%frame_buffer.addr =		getelementptr %story_handel_type, ptr %handel.address, i32 1
%frame_buffer =				load ptr, ptr %frame_buffer.addr

							store i1 true, ptr @continue_maximally

%result =					call {ptr, %yield_type} %cont_fn(ptr %frame_buffer, i1 0)
							
%result.cont_fn =			extractvalue {ptr, %yield_type} %result, 0
							store ptr %result.cont_fn, ptr %cont_fn.addr

							ret ptr %output_string.addr
}

; Returns false if story requires a choice selection or otherwise cannot continue
; it's control flow
define i1 @CanContinue(ptr %handel)
{
; TODO:
; %promise.addr =				call ptr @llvm.coro.promise(ptr %handel, i32 0, i1 false) ; TODO: Get target platform alignment
; 
; %continue_flag.addr =		getelementptr %yield_type, ptr %promise.addr, i32 1
; %continue_flag =			load i1, ptr %continue_flag.addr
; 							ret i1 %continue_flag
 							ret i1 true
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
; TODO:
; %choice_count.addr =		getelementptr %choice_list_type, ptr @choice_list, i32 0
; %choice_count =				load i32, ptr %choice_count.addr
; 
; %index_less_than =			icmp uge i32 %choice_index, 0
; %index_positive = 			icmp ult i32 %choice_index, %choice_count
; %valid_index =				and i1 %index_less_than, %index_positive
; 
; 							br i1 %valid_index, label %success, label %error
; error:
; 							call i32 @puts(ptr @error_message)
; 							ret void
; success:
; %promise.addr =				call ptr @llvm.coro.promise(ptr %handel, i32 0, i1 false) ; TODO: Get target platform alignment
; %promise.choice_index.addr =getelementptr %yield_type, ptr %promise.addr, i32 0
; 							store i32 %choice_index, ptr %promise.choice_index.addr
; %promise.continue_flag.addr=getelementptr %yield_type, ptr %promise.addr, i32 1
; 							store i1 true, ptr %promise.continue_flag.addr
; 							ret void
 							ret void
}

declare %result_type @cont_fn_proto(ptr %frame_buff, i1 %flag)
%result_type =					type { ptr, %yield_type }
%yield_type =					type { i32, i1} 
								; 0 choice_index: i32 
								; 1 continue_flag: i1


@ptr_size =						constant i32 8 ;TODO: Determine pointer size
@allignment =					constant i32 0 ;TODO: Determine data allignment

; Story

;Content Strings
@story.str_0 =					constant [ 7 x i8] c"Hello!\00"
@story.str_1 =					constant [ 2 x i8] c"a\00"

@story.choice_0.str_0 =			constant [ 7 x i8] c"Chose \00"
@story.choice_0.str_choice =	constant [ 2 x i8] c"A\00"
@story.choice_0.str_1 =			constant [11 x i8] c" the first\00"

@story.choice_1.str_0 =			constant [ 4 x i8] c"Or \00"
@story.choice_1.str_choice =	constant [ 2 x i8] c"B\00"
@story.choice_1.str_1 =			constant [12 x i8] c" the second\00"

@story.gather_0.str_0 =			constant [ 8 x i8] c"The end\00"

;Story.<knot>.<stitch>.<label>.body
@story.root.root.body =			constant [3 x ptr] [
									ptr @story.str_0, 
									ptr @story.str_1, 
									ptr @story.gather_0.str_0
								]

@story.B.str_0 =				constant [13 x i8] c"Start tunnel\00"
@story.B.str_2 =				constant [11 x i8] c"End tunnel\00"

@story.B.root.body =			constant [2 x ptr] [
									ptr @story.B.str_0,
									ptr @story.B.str_2
								]

define %result_type @__root(ptr %frame_buffer, i1 %flag) presplitcoroutine noinline
{
entry:
%ptr_size =					load i32, ptr @ptr_size
%allignment =				load i32, ptr @allignment
%id =						call token @llvm.coro.id.retcon(
									i32 %ptr_size, i32 %allignment, ptr %frame_buffer, 
									ptr @cont_fn_proto,
									ptr @malloc, ptr @free
							)

%handel =					call noalias ptr @llvm.coro.begin(token %id, ptr null)

%resume_abnormal_begin =	call i1 (...) @llvm.coro.suspend.retcon.i1(%yield_type {i32 0, i1 true})
							br i1 %resume_abnormal_begin, label %error, label %story

error:
							;TODO: Error handeling
							call i32 @puts(ptr @error_message)
							br label %destroy
destroy:
%unused =					call i1 @llvm.coro.end(ptr null, i1 true, token none)
%frame_mem =				call ptr @llvm.coro.free(token %id, ptr %handel)
							call void @free(ptr %frame_buffer)
							br label %end
end:
							ret %result_type { ptr null, %yield_type {i32 0, i1 false}}

story:
							br label %loop_0

loop_0:
%index_0 =					phi i32 [0, %story], [%inc_0, %resume.loop_0], [%inc_0, %suspend_point.loop_0]
%string_addr_0 =			getelementptr ptr, ptr @story.root.root.body, i32 %index_0
%string_0 =					load ptr, ptr %string_addr_0
							call i32 @write_string(ptr @out_stream, ptr %string_0)

%inc_0 =					add i32 %index_0, 1
%cond_0 =					icmp ule i32 %inc_0, 1
							br i1 %cond_0, label %resume.loop_0, label %cont_0
resume.loop_0:
%continue_value_0 =			load i1, ptr @continue_maximally
							br i1 %continue_value_0, label %loop_0, label %suspend_point.loop_0
suspend_point.loop_0:
%resume_abnormal_loop_0 =	call i1 (...) @llvm.coro.suspend.retcon.i1(%yield_type {i32 0, i1 true})
							br i1 %resume_abnormal_loop_0, label %error, label %loop_0
cont_0:
							;"-> B ->"
%thread_cont_0 =			call %result_type @B()
							br label %thread_0

thread_0:
%result_phi_0 =				phi %result_type [%thread_cont_0, %cont_0], [%thread_result_0, %resume.thread_0], [%thread_result_0, %suspend_point.thread_0]
%cont_fn_B =				extractvalue %result_type %result_phi_0, 0
%thread_result_0 =			call %result_type %cont_fn_B()

%thread_continue_flag =		extractvalue %result_type %thread_result_0, 1, 1

							br i1 %thread_continue_flag, label %resume.thread_0, label %story.choice_point_0
resume.thread_0:
%continue_value_1 =			load i1, ptr @continue_maximally
							br i1 %continue_value_1, label %thread_0, label %suspend_point.thread_0
suspend_point.thread_0:
%choice_count_thread_0 =	extractvalue %result_type %thread_result_0, 1, 0
%continue_thread_0 =		extractvalue %result_type %thread_result_0, 1, 1
%yield_thread_0.addr =		alloca %yield_type

%yield_thread_0_count.addr=	getelementptr %result_type, ptr %yield_thread_0.addr, i32 1, i32 0
%yield_thread_0_flag.addr =	getelementptr %result_type, ptr %yield_thread_0.addr, i32 1, i32 1
							store i32 %choice_count_thread_0, ptr %yield_thread_0_count.addr
							store i1 %continue_thread_0, ptr %yield_thread_0_flag.addr

%yield_thread_0 =			load %yield_type, ptr %yield_thread_0.addr
%resume_abnormal_thread_0 =	call i1 (...) @llvm.coro.suspend.retcon.i1(%yield_type %yield_thread_0)
							br i1 %resume_abnormal_thread_0, label %error, label %thread_0

story.choice_point_0:
%choice_count_choice_point_0 =		extractvalue %result_type %thread_result_0, 1, 0
%choice_count_sum =					add i32 %choice_count_choice_point_0, 2

%yield_choice_point_0.addr =		alloca %yield_type
%yield_choice_point_0_count.addr=	getelementptr %result_type, ptr %yield_choice_point_0.addr, i32 1, i32 0
%yield_choice_point_0_flag.addr =	getelementptr %result_type, ptr %yield_choice_point_0.addr, i32 1, i32 1
									store i32 %choice_count_sum, ptr %yield_choice_point_0_count.addr
									store i1 false, ptr %yield_choice_point_0_flag.addr

%yield_choice_point_0 =				load %yield_type, ptr %yield_choice_point_0.addr
%resume_abnormal_choice_point_0 =	call i1 (...) @llvm.coro.suspend.retcon.i1(%yield_type %yield_choice_point_0)
									br i1 %resume_abnormal_choice_point_0, label %error, label %resume_story.choice_point_0

;TODO: Move choices into functions
resume_story.choice_point_0:
;story.choice_0:					;"* Chose [A] the first"
;							;"Chose "
;							call i32 @write_string(ptr @out_stream, ptr @story.choice_0.str_0)
;
;							;" the first"
;							call i32 @write_string(ptr @out_stream, ptr @story.choice_0.str_1)
;
;							br label %story.gather_0
;
;;TODO: Move choices into functions
;story.choice_1:					;"* Or [B] the second"
;							;"Or "
;							call i32 @write_string(ptr @out_stream, ptr @story.choice_1.str_0)
;
;							;" the second"
;%str_choice_1.1 =			getelementptr [0 x i8], ptr @story.choice_1.str_1, i32 0, i32 0
;							call i32 @write_string(ptr @out_stream, ptr @story.choice_1.str_1)
;
							br label %story.gather_0

story.gather_0:					;"-"
							br label %loop_1
loop_1:
%index_1 =					phi i32 [2, %story.gather_0], [%inc_1, %resume.loop_1], [%inc_1, %suspend_point.loop_1]
%string_addr_1 =			getelementptr ptr, ptr @story.root.root.body, i32 %index_1
%string_1 =					load ptr, ptr %string_addr_1
							call i32 @write_string(ptr @out_stream, ptr %string_1)

%inc_1 =					add i32 %index_1, 1
%cond_1 =					icmp ule i32 %inc_1, 2
							br i1 %cond_1, label %resume.loop_1, label %cont_1
resume.loop_1:
%continue_value_2 =			load i1, ptr @continue_maximally
							br i1 %continue_value_2, label %loop_1, label %suspend_point.loop_1
suspend_point.loop_1:
%resume_abnormal_loop_1 =	call i1 (...) @llvm.coro.suspend.retcon.i1(%yield_type {i32 0, i1 true})
							br i1 %resume_abnormal_loop_1, label %error, label %loop_1
cont_1:
%resume_abnormal_gather_0 =	call i1 (...) @llvm.coro.suspend.retcon.i1(%yield_type {i32 0, i1 false})
							br label %destroy
}

define %result_type @B(ptr %frame_buff, i1 %flag)  presplitcoroutine noinline
{
entry:
%ptr_size =					load i32, ptr @ptr_size
%allignment =				load i32, ptr @allignment
%frame_buffer =				call ptr @malloc(i32 %ptr_size)
%id =						call token @llvm.coro.id.retcon(
									i32 %ptr_size, i32 %allignment, ptr %frame_buffer, 
									ptr @cont_fn_proto,
									ptr @malloc, ptr @free
							)

%handel =					call noalias ptr @llvm.coro.begin(token %id, ptr null)

%resume_abnormal_begin =	call i1 (...) @llvm.coro.suspend.retcon.i1(%yield_type {i32 0, i1 true})
							br i1 %resume_abnormal_begin, label %error, label %story

error:
							;TODO: Error handeling
							call i32 @puts(ptr @error_message)
							br label %destroy
destroy:
%unused =					call i1 @llvm.coro.end(ptr null, i1 true, token none)
%frame_mem =				call ptr @llvm.coro.free(token %id, ptr %handel)
							call void @free(ptr %frame_buffer)
							br label %end
end:
							ret %result_type { ptr null, %yield_type {i32 0, i1 false}}

story:
							br label %loop_0

loop_0:
%index_0 =					phi i32 [0, %story], [%inc_0, %resume.loop_0], [%inc_0, %suspend_point.loop_0]
%string_addr_0 =			getelementptr ptr, ptr @story.B.root.body, i32 %index_0
%string_0 =					load ptr, ptr %string_addr_0
							call i32 @write_string(ptr @out_stream, ptr %string_0)

%inc_0 =					add i32 %index_0, 1
%cond_0 =					icmp ule i32 %inc_0, 1
							br i1 %cond_0, label %resume.loop_0, label %B.cont_0
resume.loop_0:
%continue_value_0 =			load i1, ptr @continue_maximally
							br i1 %continue_value_0, label %loop_0, label %suspend_point.loop_0
suspend_point.loop_0:
%resume_abnormal_loop_0 =	call i1 (...) @llvm.coro.suspend.retcon.i1(%yield_type {i32 0, i1 true})
							br i1 %resume_abnormal_loop_0, label %error, label %loop_0
B.cont_0:
							br label %story.choice_point_0

story.choice_point_0:
%resume_abnormal_choice_point_0 =	call i1 (...) @llvm.coro.suspend.retcon.i1(%yield_type {i32 0, i1 false})
							br i1 %resume_abnormal_choice_point_0, label %error, label %B.cont_0
cont_0:

%resume_abnormal_gather_0 =	call i1 (...) @llvm.coro.suspend.retcon.i1(%yield_type {i32 0, i1 true})
							br label %destroy
}
