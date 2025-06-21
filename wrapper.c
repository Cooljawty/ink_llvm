#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include "wrapper.h"

void print_status(Story story)
{
	printf("Status:%s\nContinue?: %s\nChoice count: %i\n", 
			story == NULL ? "Ended" : "Unfinished", 
			CanContinue(story) ? "True":"False"	,
			ChoiceCount(story)
	);
}

void string_test()
{
	string* test_str = new_string();
	write_string(test_str, "Hello!");

	char* read_buf = malloc(1);
	read_buf[0] = '\0';
	read_string(test_str, read_buf);
	printf("Test string: \'%s\'\n", read_buf);

	printf("Flushing string:\n");
	flush_string(test_str);
	printf("\n");

	free_string(test_str);
}

string* tmp(){ string* tmp = new_string(); return tmp; }
int main() 
{
	//string_test();

	string* prev_string = NULL;

	printf("Initilizing:\n");
	Story story = NewStory(); 
	do{
		while(CanContinue(story))
		{
			printf("Stepping..\n");
			
			string* str = Step(story);
			//string* str = ContinueMaximally(story);

			printf("paused..");
			getchar();

			printf("\'%s\'\n", str->buffer);
			//if(prev_string != NULL) free_string(prev_string);
			prev_string = str;
			
		}

		if(ChoiceCount(story) > 0)
		{
			unsigned int choice = 0;
			print_status(story);
			printf("Chose choice: ");
			scanf("%u", &choice);
			ChooseChoiceIndex(story, choice);
		}

	} while(CanContinue(story) || ChoiceCount(story) > 0);

	if(prev_string != NULL) free_string(prev_string);

	return 0;
}
